use std::net;
use std::io;

pub fn listen_for_handshake(code: &[u8], addr: &[u8]) {
    let socket = match net::UdpSocket::bind("0.0.0.0:5514") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e),
    };

    let mut buf = [0; 2048];
    loop {
        match socket.recv_from(&mut buf) {
            Ok((amt, src)) => {
                let recv_code = &buf[0..amt];

                if recv_code != code {
                    println!("ignore invalid handshake {:?}:{:?}", code, recv_code);
                    continue;
                }

                println!("handshake received from {}", src);
                socket.send_to(addr, src).expect("failed to send addr");
            }
            Err(e) => println!("couldn't recieve a datagram: {}", e),
        }
    }
}

/// do_handshake 使用指定的 `code` 进行 upd 广播, 在成功收到文件地址后, 返回文件下载地址
/// 或者在超时后返回错误
pub fn do_handshake(code: &[u8], addr_buf: &mut [u8]) -> io::Result<usize> {
    let socket = try!(net::UdpSocket::bind("0.0.0.0:0"));
    try!(socket.set_broadcast(true));

    socket
        .send_to(code, "255.255.255.255:5514")
        .expect("failed to send handshake");

    socket.recv_from(addr_buf).map(|(amt, _)| amt)
}
