#![allow(clippy::type_complexity)]

use super::http_proxy_stream::HttpProxyStream;
use http::Uri;
use hyper::service::Service;
use std::{
    future::Future,
    io::IoSlice,
    pin::Pin,
    task::{self, Poll},
};
use tokio::{
    io::AsyncBufReadExt,
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("lookup error")]
    LookupError,
    #[error("connect error: {0}")]
    ConnectError(#[from] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct HttpProxyConnector {
    proxy_uri: Uri,
    connect: bool,
}

impl HttpProxyConnector {
    pub fn new(proxy_uri: Uri) -> Self {
        Self {
            proxy_uri,
            connect: false,
        }
    }

    /// When `connect` is true, it will issue the HTTP CONNECT verb to establish
    /// a connection with the host behind the proxy.
    pub fn new_with_connect(proxy_uri: Uri, connect: bool) -> Self {
        Self { proxy_uri, connect }
    }

    async fn call_async(&mut self, dst: Uri) -> std::result::Result<HttpProxyStream, Error> {
        let port = self.proxy_uri.port_u16().unwrap_or(3128);
        let host = self.proxy_uri.host().ok_or(Error::LookupError)?;
        let stream = TcpStream::connect((host, port))
            .await
            .map_err(Error::ConnectError)?;

        if self.connect {
            let mut stream = BufReader::new(stream);

            let host = dst.host().unwrap_or_default();
            let port = dst.port();
            let port = if let Some(ref port) = port {
                port.as_str()
            } else {
                "443"
            };

            let connect_verb: &[_] = &[
                // "CONNECT {host}:{port} HTTP/1.1\r\n"
                // "Host: {host}:{port}\r\n"
                // "\r\n"
                IoSlice::new(b"CONNECT "),
                IoSlice::new(host.as_bytes()),
                IoSlice::new(b":"),
                IoSlice::new(port.as_bytes()),
                IoSlice::new(b" HTTP/1.1\r\nHost: "),
                IoSlice::new(host.as_bytes()),
                IoSlice::new(b":"),
                IoSlice::new(port.as_bytes()),
                IoSlice::new(b"\r\n\r\n"),
            ];
            stream.write_vectored(connect_verb).await?;

            loop {
                // TODO: parse the protocol to ensure we received an OK
                let mut line = String::new();
                stream.read_line(&mut line).await?;
                let line = line.trim();
                if line.is_empty() {
                    // saw "\r\n\r\n", header finished.
                    break;
                }
            }

            // TODO: need to handle potential extra data read by the BufReader
            // can be accessed via BufReader::buffers().
            if !stream.buffer().is_empty() {
                todo!("handle received, but buffered data from upstream server");
            }

            return Ok(HttpProxyStream::new_connected(stream.into_inner()));
        }

        Ok(HttpProxyStream::new(stream))
    }
}

impl Service<Uri> for HttpProxyConnector {
    type Response = HttpProxyStream;
    type Error = Error;
    // We can't "name" an `async` generated future.
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut task::Context<'_>) -> Poll<Result<(), Error>> {
        // This connector is always ready.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let mut this = self.clone();
        Box::pin(async move { this.call_async(dst).await })
    }
}
