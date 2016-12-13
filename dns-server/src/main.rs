#![recursion_limit = "1024"]

#[macro_use] extern crate error_chain;

mod packet;

use std::net::UdpSocket;
use packet::error::*;


fn bind_udp() -> Result<()> {
    let mut socket = UdpSocket::bind("0.0.0.0:53")?;

    // read from the socket
    let mut buf = [0; 4096];
    let (amt, src) = socket.recv_from(&mut buf)?;

    let p = packet::Message::read(&buf[..amt])?;

    println!("{:?}", p);

    // send a reply to the socket we received data from
    Ok(())
}


fn main() {
    bind_udp().unwrap();
}
