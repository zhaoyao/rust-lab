#[macro_use]
extern crate futures;

extern crate tokio;
extern crate tokio_core;
extern crate trust_dns;
extern crate byteorder;
extern crate bytes;

#[macro_use]
extern crate log;
extern crate env_logger;

pub mod socks5;
use socks5::Address;

mod util;

use futures::future;

use trust_dns::client::{ClientFuture, BasicClientHandle, ClientHandle};
use trust_dns::udp::{UdpClientStream};
use trust_dns::op::{Message, ResponseCode};
use trust_dns::rr::{DNSClass, RData, RecordType};
use trust_dns::rr::domain;

use socks5::{HandshakeReq, HandshakeRep, RelayRequest, RelayRep};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;

fn main() {
    env_logger::init();

    let core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let dns = "8.8.8.8:53".parse().unwrap();
    let (stream, sender) = UdpClientStream::new(dns, &handle.clone());
    let dns_client = ClientFuture::new(stream, sender, &handle.clone(), None);

    // Bind the server's socket.
    let addr = "127.0.0.1:12345".parse().unwrap();
    let listener = TcpListener::bind(&addr).expect("unable to bind TCP listener");

    // Pull out a stream of sockets for incoming connections
    let server = listener
        .incoming()
        .for_each(|sock| {
            handle_socks5_client(sock, dns_client.clone())
        }).map_err(|err| {
            warn!("accept error: {:?}", err);
            ()
        });

    // Start the Tokio runtime
    info!("starting server");
    tokio::run(server);
}


fn boxed_future<T, E, F>(f: F) -> Box<Future<Item = T, Error = E> + Send>
    where F: Future<Item = T, Error = E> + Send + 'static
{
    Box::new(f)
}

fn handle_socks5_client(sock: TcpStream, dns_client: BasicClientHandle) -> io::Result<()> {
    println!("new connection");

    let (r, w) = sock.split();

    let handle_client = futures::lazy(|| Ok(sock.split()))
        .and_then(handshake)
        .and_then(authenticate)
        .and_then(read_request)
        .and_then(|(r, w, req)| {
            match req.cmd {
                socks5::CMD_CONNECT => {
                    let resolve_dst_addr = match req.addr {
                        Address::Name(name, port) => {
                            boxed_future(
                                resolve_domain_addr(name, dns_client)
                                .map(|ip| SocketAddr(ip, port)))
                        }

                        Address::SocketAddr(addr) => {
                            boxed_future(future::ok(addr))
                        }
                    };

                    resolve_dst_addr
                        .then(|res| {
                            match res {
                                Ok(addr) => {
                                    info!("connecting dst addr: {:?}", addr);
                                    boxed_future(future::ok((r, w)))

                                }
                                Err(err) => {
                                    let f = reply_error(w, 
                                                        socks5::RepCode::HostUnreachable,
                                                        &format!("invalid dst addr {:?}: {:?}", req.addr, err))
                                        .then(|_| Err(err));
                                    boxed_future(f)
                                }
                            }
                        })

                    boxed_future(future::ok((r, w)))
                    

                    //connect_remote(req.addr)

                }
                cmd => {
                    let f = reply_error( w, socks5::RepCode::CommandNotSupported, "only support CMD_CONNECT")
                        .map(|w| (r, w));
                        .then(|_| Err(ioOtherErr(&format!("unsupported command: {}", cmd))));

                    boxed_future(f)
                }
            }
        });

    tokio::spawn(handle_client.then(|res| match res {
        Ok(..) => Ok(()),

        Err(err) => {
            println!("failed to handle client {:?}", err);
            Err(())
        }
    }));

    Ok(())
}

fn resolve_domain_addr(name: String, dns: BasicClientHandle) 
    -> impl Future<Item=IpAddr, Error=io::Error>
{
    
    let parsed_domain = domain::Name::parse(&name, None);
    if parsed_domain.is_err() {
        return boxed_future(future::err(ioOtherErr(format!("invalid domain name: {}", name).as_str())))
    }

    let f = dns.query(parsed_domain.unwrap(), DNSClass::IN, RecordType::A)
        .map_err(|e| ioOtherErr(&format!("dns error: {:?}", e)))
        .and_then(|r| {

            if r.response_code() != ResponseCode::NoError {
                return Err(ioOtherErr(
                        &format!("resolution error: {}", r.response_code())))
            }

            let resolved_ip_addr = r.answers().iter().filter_map(|ans| {
                match *ans.rdata() {
                    RData::A(addr) => Some(IpAddr::V4(addr)),
                    RData::AAAA(addr) => Some(IpAddr::V6(addr)),
                    _ => None
                }
            }).next();

            match resolved_ip_addr {
                Some(addr) => Ok(addr),
                None => Err(ioOtherErr(&format!("domain {} not resolved", name)))
            }

        });

    Box::new(f)
}


/*
fn connect_remote<R, W>(param: (R, W, socks5::Address))
    where W: AsyncWrite 
    -> impl Future<Item = ((R, W), (R, W)), Error = io::Error> 
{
    use socks5::Address;

    let (cr, cw, addr) = param;

    tokio::spawn(


    match addr {
        Address::Name(domain, port) => {
            tcp::connect()
        }
        Address::SocketAddr(SocketAddrV4(addr)) => {}
        Address::SocketAddr(SocketAddrV6(addr)) => {}

    }


}
*/

fn ioOtherErr(msg: &str) -> io::Error {
    io::Error::new(io::ErrorKind::Other, msg)
}

/// 向客户端应答错误
fn reply_error<W>(w: W, code: socks5::RepCode, msg: &str) 
    -> impl Future<Item = W, Error = io::Error> 
    where W: AsyncWrite + Send + 'static,
{
    let f = RelayRep::new(code, None)
            .write_to(w);
    Box::new(f)
}

fn handshake<R, W>(param: (R, W)) 
    -> impl Future<Item = (R, W, u8), Error = io::Error>

    where R: AsyncRead + Send + 'static,
          W: AsyncWrite + Send + 'static,
{
    let (r, w) = param;

    let f= HandshakeReq::read_from(r)
        .map(|(r, req)| (r, w, req))
        .and_then(negotiate_auth_methods);

    Box::new(f)
}

fn negotiate_auth_methods<R, W>(param: (R, W, HandshakeReq)) 
    -> impl Future<Item = (R, W, u8), Error = io::Error>

    where R: AsyncRead + Send + 'static,
          W: AsyncWrite + Send + 'static,
{
    let (r, w, req) = param;

    trace!("authenticating client: {:?}", req);

    if !req.methods.contains(&socks5::METH_NO_AUTH) {
        let f = HandshakeRep::new(socks5::METH_METHOD_NOT_ACCEPTABLE)
            .write_to(w)
            .then(|_| Err(ioOtherErr("Only accept NO_AUTH")));

        return Box::new(f)
    }

    let f = socks5::HandshakeRep::new(socks5::METH_NO_AUTH)
        .write_to(w)
        .and_then(|w| Ok((r, w, socks5::METH_NO_AUTH)));

    Box::new(f)
}

fn authenticate<R, W>(param: (R, W, u8)) 
    -> impl Future<Item = (R, W), Error = io::Error>
    where R: AsyncRead + Send + 'static,
          W: AsyncWrite + Send + 'static,
{

    future::ok((param.0, param.1))
}

fn read_request<R, W>(param: (R, W)) -> Box<Future<Item = (R, W, socks5::RelayRequest), Error = io::Error> + Send> 
    where R: AsyncRead + Send + 'static,
          W: AsyncWrite + Send + 'static,
{
    let (r, w) = param;

    let f = socks5::RelayRequest::read_from(r)
        .then(move |res| {

            match res {
            
                Ok((r, req)) => {
                    info!("recv relay request: {:?}", req);
                    boxed_future(future::ok((r, w, req)))
                }
            
                Err(err) => {
                    let code = err.clone().rep;
                    boxed_future(socks5::RelayRep::new(code, None)
                            .write_to(w)
                            .then(move |_| Err(From::from(err))))
                }
            }

        });
    Box::new(f)
}
