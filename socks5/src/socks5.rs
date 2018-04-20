use byteorder::BigEndian;
use bytes::{BufMut, Bytes, BytesMut};
use futures::Future;
use std::error;
use std::fmt;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::u8;
use tokio::io::read_exact;
use tokio::prelude::*;
use util::{write_bytes, WriteBytes};

pub const VERSION: u8 = 5;

pub const METH_NO_AUTH: u8 = 0x00;
pub const METH_GSSAPI: u8 = 0x01;
pub const METH_USER_PASS: u8 = 0x02;
pub const METH_METHOD_NOT_ACCEPTABLE: u8 = 0xff;

pub const CMD_CONNECT: u8 = 1;
pub const CMD_BIND: u8 = 2;
pub const CMD_UDP_ASSOCIATE: u8 = 3;

pub const ATYP_IPV4: u8 = 1;
pub const ATYP_DOMAIN: u8 = 3;
pub const ATYP_IPV6: u8 = 4;

pub type BoxIoFuture<T> = Box<Future<Item = T, Error = io::Error> + Send>;

/// SOCKS5 protocol error
#[derive(Clone)]
pub struct Error {
    pub rep: RepCode,
    pub message: String,
}

impl Error {
    fn new(rep: RepCode, msg: &str) -> Error {
        Error {
            rep: rep,
            message: msg.to_string(),
        }
    }
}

impl fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        &self.message[..]
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::new(RepCode::ServerFailure,
                   <io::Error as error::Error>::description(&err))
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> io::Error {
        io::Error::new(io::ErrorKind::Other, err.message)
    }
}

/// Socks5 client Auth Handshake Request
#[derive(Clone, Debug)]
pub struct HandshakeReq {
    pub methods: Vec<u8>,
}

impl HandshakeReq {
    /// Creates a handshake request
    pub fn new(methods: Vec<u8>) -> HandshakeReq {
        HandshakeReq { methods: methods }
    }

    pub fn read_from<R: AsyncRead + 'static>(
        r: R,
    ) -> impl Future<Item = (R, HandshakeReq), Error = io::Error> {

        let fut = read_exact(r, [0u8, 0u8])
            .and_then(|(r, buf)| {
                let ver = buf[0];
                let nmet = buf[1];

                if ver != VERSION {
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Invalid Socks5 version",
                    ));
                }

                Ok((r, nmet))
            })
            .and_then(|(r, nmet)| {
                read_exact(r, vec![0u8; nmet as usize])
                    .and_then(|(r, methods)| Ok((r, HandshakeReq { methods: methods })))
            });

        Box::new(fut)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HandshakeRep {
    pub method: u8,
}

impl HandshakeRep {
    pub fn new(method: u8) -> Self {
        HandshakeRep { method }
    }

    pub fn write_to<W: AsyncWrite + Send>(self, w: W) -> WriteBytes<W, Bytes> {
        let mut buf = BytesMut::with_capacity(2);
        buf.put_slice(&[VERSION, self.method]);
        write_bytes(w, buf.freeze())
    }
}

#[derive(Debug)]
pub struct RelayRequest {
    pub cmd: u8,
    //pub
    pub addr: Address,
}

///
/// +----+-----+-------+------+----------+----------+
/// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
/// +----+-----+-------+------+----------+----------+
/// | 1  |  1  | X'00' |  1   | Variable |    2     |
/// +----+-----+-------+------+----------+----------+
///  Where:
///   o  VER    protocol version: X'05'
///   o  CMD
///   o  CONNECT X'01'
///      o  BIND X'02'
///      o  UDP ASSOCIATE X'03'
///   o  RSV    RESERVED
///   o  ATYP   address type of following address
///      o  IP V4 address: X'01'
///      o  DOMAINNAME: X'03'
///      o  IP V6 address: X'04'
///   o  DST.ADDR       desired destination address
///   o  DST.PORT desired destination port in network octet order
impl RelayRequest {
    pub fn read_from<R: AsyncRead + Send + 'static>(
        r: R,
    ) -> impl Future<Item = (R, RelayRequest), Error = Error> + Send {
        let f = read_exact(r, [0u8; 3])
            .map_err(From::from)
            .and_then(|(r, buf)| {
                if buf[1] != CMD_CONNECT {
                    Err(Error::new(
                        RepCode::CommandNotSupported,
                        "Unsupported command",
                    ))
                } else {
                    Ok((r, buf[1]))
                }
            })
            .and_then(|(r, cmd)| {
                Address::read_from(r).map(move |(r, addr)| {
                    (
                        r,
                        RelayRequest {
                            cmd: cmd,
                            addr: addr,
                        },
                    )
                })
            });

        Box::new(f)
    }
}

#[derive(Debug, Clone)]
pub enum RepCode {
    Success = 0x00,
    ServerFailure = 0x01,
    RuleFailure = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TtlExpired = 0x06,
    CommandNotSupported = 0x07,
    AddrTypeNotSupported = 0x08,
}

#[derive(Debug)]
pub struct RelayRep {
    pub rep: RepCode,
    pub addr: Option<Address>,
}

const RELAY_REP_RSV: u8 = 0;

impl RelayRep {
    pub fn new(rep: RepCode, addr: Option<Address>) -> Self {
        RelayRep {
            rep: rep,
            addr: addr,
        }
    }

    pub fn write_to<W: AsyncWrite + Send>(self, w: W) -> WriteBytes<W, Bytes> {
        let addr = self.addr
            .unwrap_or(Address::SocketAddr(SocketAddr::V4(SocketAddrV4::new(
                Ipv4Addr::new(0, 0, 0, 0),
                0,
            ))));

        let len = 1/*ver*/ +1/*rep*/+1/*rsv*/ +1/*atyp*/+addr.len();
        let mut buf = BytesMut::with_capacity(len);

        buf.put_slice(&[VERSION, self.rep as u8, RELAY_REP_RSV, addr.atype()]);
        addr.write_to(&mut buf);

        write_bytes(w, buf.freeze())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Address {
    Name(String, u16),
    SocketAddr(SocketAddr),
}

impl Address {
    pub fn atype(&self) -> u8 {
        match self {
            Address::Name(..) => ATYP_DOMAIN,
            Address::SocketAddr(SocketAddr::V4(..)) => ATYP_IPV4,
            Address::SocketAddr(SocketAddr::V6(..)) => ATYP_IPV6,
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Address::Name(name, _) => name.len() + 2,
            Address::SocketAddr(SocketAddr::V4(..)) => 6,
            Address::SocketAddr(SocketAddr::V6(..)) => 18,
        }
    }

    pub fn read_from<R: AsyncRead + Send + 'static>(
        r: R,
    ) -> impl Future<Item = (R, Address), Error = Error> {
        let f = read_exact(r, [0u8])
            .map_err(From::from)
            .and_then(|(r, buf)| match buf[0] {
                ATYP_IPV4 => read_v4_addr(r),
                ATYP_IPV6 => read_v6_addr(r),
                ATYP_DOMAIN => read_domain_name(r),
                n => Box::new(future::err(Error::new(
                    RepCode::AddrTypeNotSupported,
                    format!("Unsupported address type {:?}", n).as_str(),
                ))),
        });

        Box::new(f)
    }

    pub fn write_to<B: BufMut>(self, buf: &mut B) {
        match self {
            Address::Name(name, port) => {
                buf.put_u8(ATYP_DOMAIN);
                buf.put_slice(name.as_bytes());
                buf.put_u16::<BigEndian>(port);
            }
            Address::SocketAddr(SocketAddr::V4(addr_v4)) => {
                buf.put_u8(ATYP_IPV4);
                buf.put_slice(&addr_v4.ip().octets()[..]);
                buf.put_u16::<BigEndian>(addr_v4.port());
            }
            Address::SocketAddr(SocketAddr::V6(..)) => {}
        }
    }

    /*
    pub fn atyp(self) -> u8 {
        match self {
            Name(..) => v5::ATYP_DOMAIN,
            SocketAddr(_addr: SocketAddrV4)  => v5::ATYP_IPV4,
            SocketAddr(_addr: SocketAddrV6)  => v5::ATYP_IPV6,
            _ => 0,
        }
    }
    */
}

pub fn read_v4_addr<R: AsyncRead + Send + 'static>(
    r: R,
) -> Box<Future<Item = (R, Address), Error = Error> + Send> {
    let f = read_exact(r, [0u8, 6])
        .map_err(From::from)
        .map(|(r, buf)| {
            let addr = Ipv4Addr::new(buf[0], buf[1], buf[2], buf[3]);
            let port = ((buf[4] as u16) << 8) | (buf[5] as u16);
            let addr = SocketAddrV4::new(addr, port);

            (r, Address::SocketAddr(SocketAddr::V4(addr)))
        });

    Box::new(f)
}

pub fn read_v6_addr<R: AsyncRead + Send + 'static>(
    r: R,
) -> Box<Future<Item = (R, Address), Error = Error> + Send> {
    let f = read_exact(r, [0u8, 18])
        .map(|(r, buf)| {
            let a = ((buf[0] as u16) << 8) | (buf[1] as u16);
            let b = ((buf[2] as u16) << 8) | (buf[3] as u16);
            let c = ((buf[4] as u16) << 8) | (buf[5] as u16);
            let d = ((buf[6] as u16) << 8) | (buf[7] as u16);
            let e = ((buf[8] as u16) << 8) | (buf[9] as u16);
            let f = ((buf[10] as u16) << 8) | (buf[11] as u16);
            let g = ((buf[12] as u16) << 8) | (buf[13] as u16);
            let h = ((buf[14] as u16) << 8) | (buf[15] as u16);
            let addr = Ipv6Addr::new(a, b, c, d, e, f, g, h);
            let port = ((buf[16] as u16) << 8) | (buf[17] as u16);
            let addr = SocketAddrV6::new(addr, port, 0, 0);
            (r, Address::SocketAddr(SocketAddr::V6(addr)))
        })
        .map_err(|err| Error::new(RepCode::ServerFailure, error::Error::description(&err)));

    Box::new(f)
}

pub fn read_domain_name<R: AsyncRead + Send + 'static>(
    r: R,
) -> Box<Future<Item = (R, Address), Error = Error> + Send> {
    let f = read_exact(r, [0u8])
        .and_then(|(r, buf)| {
            // read var-length doamin & 2-byte port
            read_exact(r, vec![0u8; buf[0] as usize + 2])
        })
        .and_then(|(r, buf)| {
            let len = buf.len();
            if len == 2 {
                return future::err(other("0 address length"));
            }

            let name_slice = &buf[0..len - 2];
            let name = match String::from_utf8(name_slice.to_vec()) {
                Ok(s) => s,
                Err(err) => return future::err(other(error::Error::description(&err))),
            };

            let port = ((buf[len - 2] as u16) << 8) | (buf[len - 1] as u16);
            future::ok((r, Address::Name(name, port)))
        })
        .map_err(|err| Error::new(RepCode::ServerFailure, error::Error::description(&err)));

    Box::new(f)
}

fn other(desc: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, desc)
}
