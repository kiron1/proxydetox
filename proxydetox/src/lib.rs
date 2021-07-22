pub mod auth;
pub mod client;
pub mod detox;
pub mod io;
pub mod net;

use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::Path;

use hyper::body::Buf;

pub fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub async fn http_file(uri: http::Uri) -> std::io::Result<String> {
    let client = hyper::Client::new();
    let res = client
        .get(uri)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "GET"))?;
    let body = hyper::body::aggregate(res)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "aggregate"))?;
    let mut buffer = String::new();
    body.reader().read_to_string(&mut buffer)?;
    Ok(buffer)
}
