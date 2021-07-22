use hyper::body::Buf;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use tokio::net::TcpStream;
use tracing_attributes::instrument;

#[instrument]
pub async fn dial(uri: &http::Uri) -> tokio::io::Result<TcpStream> {
    match (uri.host(), uri.port_u16()) {
        (Some(host), Some(port)) => TcpStream::connect((host, port)).await,
        (_, _) => Err(tokio::io::Error::new(
            tokio::io::ErrorKind::AddrNotAvailable,
            "invalid URI",
        )),
    }
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
