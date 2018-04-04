use std::io::{self, Write};

use tokio_core;

use futures::Future;
use futures::stream::Stream;

use hyper::Uri;
use hyper::Client;
use hyper::header::ContentLength;


pub fn download(u: &str, p: &str) {
    let url = u.parse::<Uri>().unwrap();

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();
    let client = Client::new(&handle);


    let work = client
        .get(url)
        .and_then(|res| {
            println!("Response: {}", res.status());
            println!("Headers: \n{}", res.headers());

            let len = res.headers()
                .get::<ContentLength>()
                .map(|cl| cl.0)
                .unwrap_or(0u64);

            let mut downloaded = 0usize;

            res.body().for_each(move |chunk| {
                downloaded += chunk.len();
                print!("\r{}/{}", downloaded, len);

                Ok(())
            })
        })
        .map(|_| {
            println!("\n\nDone.");
        });

    core.run(work).unwrap();
}
