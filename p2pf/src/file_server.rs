use std::net::SocketAddr;

use futures::{Future, Sink};
use futures::sync::{mpsc, oneshot};

use hyper::{Chunk, Result, StatusCode};
use hyper::error::Error;
use hyper::header::ContentLength;
use hyper::server::{Http, Request, Response, Service};
use std::fs::File;
use std::io::{self, Read};
use std::thread;

static NOTFOUND: &[u8] = b"Not Found";

pub struct FileServer<'a> {
    path: &'a str,
}

impl<'a> FileServer<'a> {
    fn new(p: &str) -> FileServer {
        FileServer { path: p.clone() }
    }

    pub fn run(p: &str, addr: &str) {
        let s = p.to_string();
        let addr1 = addr.parse().unwrap();

        thread::spawn(move || {
            Http::new()
                .bind(&addr1, move || Ok(FileServer::new(&s)))
                .unwrap()
                .run()
                .expect("failed to run file server")
        });
    }
}

impl<'a> Service for FileServer<'a> {
    type Request = Request;
    type Response = Response;
    type Error = Error;
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;

    fn call(&self, _: Request) -> Self::Future {
        let rx = send_file(self.path);
        Box::new(rx.map_err(
            |e| Error::from(io::Error::new(io::ErrorKind::Other, e)),
        ))
    }
}

fn send_file(p: &str) -> oneshot::Receiver<Response> {
    // Stream a large file in chunks. This requires a
    // little more overhead with two channels, (one for
    // the response future, and a second for the response
    // body), but can handle arbitrarily large files.
    //
    // We use an artificially small buffer, since we have
    // a small test file.

    let (tx, rx) = oneshot::channel();

    let mut file = match File::open(p) {
        Ok(f) => f,
        Err(_) => {
            tx.send(
                Response::new()
                    .with_status(StatusCode::NotFound)
                    .with_header(ContentLength(NOTFOUND.len() as u64))
                    .with_body(NOTFOUND),
            ).expect("Send error on open");
            return rx;
        }
    };

    thread::spawn(move || {
        let (mut tx_body, rx_body) = mpsc::channel(1);
        let res = Response::new()
            .with_header(ContentLength(file.metadata().unwrap().len()))
            .with_body(rx_body);
        tx.send(res).expect("Send error on successful file read");
        let mut buf = [0u8; 4096];
        loop {
            match file.read(&mut buf) {
                Ok(n) => {
                    if n == 0 {
                        // eof
                        tx_body.close().expect("panic closing");
                        return;
                    } else {
                        let chunk: Chunk = buf[0..n].to_vec().into();
                        match tx_body.send(Ok(chunk)).wait() {
                            Ok(t) => {
                                tx_body = t;
                            }
                            Err(_) => {
                                break;
                            }
                        };
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    });

    rx
}
