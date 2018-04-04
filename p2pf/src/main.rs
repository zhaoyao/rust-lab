extern crate clap;
extern crate futures;
extern crate hyper;
extern crate tokio_core;
mod handshake;
mod download;
mod file_server;

use clap::{App, Arg};
use std::str;
use handshake::*;
use download::*;
use file_server::FileServer;
use std::net;


fn main() {
    let matches = App::new("p2pf")
        .version("1.0")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("target file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("code")
                .short("c")
                .long("code")
                .takes_value(true)
                .value_name("CODE")
                .help("file code"),
        )
        .get_matches();

    match matches.value_of("file") {
        Some(path) => {
            //serve_file(path)
            //let addr = "127.0.0.1:3000".parse().unwrap();
            //let server = Http::new().bind(&addr, || Ok(HelloWorld)).unwrap();
            //server.run().unwrap();
            let addr_str = "127.0.0.1:1234";
            FileServer::run(path, addr_str);


            listen_for_handshake("1234".as_bytes(), addr_str.as_bytes())
        }
        None => {}
    }

    match matches.value_of("code") {
        Some(code) => {
            //receive_file_by_code(code)
            let mut buf = [0; 2048];
            let addr = do_handshake(code.as_bytes(), &mut buf).map(|i| str::from_utf8(&buf[0..i]));

            match addr {
                Ok(addr) => {
                    let url = format!("http://{}", addr.unwrap());
                    println!("got address: {}", url);
                    download(url.as_str(), "");
                }

                Err(err) => {
                    println!("handshake err: {}", err);
                }
            }
        }
        None => {}
    }
}
