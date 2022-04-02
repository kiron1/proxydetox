use crate::HttpConnectStream;
use http::{header::HOST, Request, StatusCode};
use hyper::{body::Bytes, client::conn, Body};
use std::{future::Future, io::Cursor, pin::Pin};
use tokio::{io::AsyncReadExt, net::TcpStream};
use tower::ServiceExt;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid proxy URI: {0}")]
    InvalidProxyUri(http::Uri),
    #[error("Invalid CONNECT URI: {0}")]
    InvalidConnectUri(http::Uri),
    #[error("IO error: {0}")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("HTTP error: {0}")]
    Http(
        #[from]
        #[source]
        http::Error,
    ),
    #[error("Unexpected HTTP status code: {0}")]
    UnexpectedHttpStatusCode(http::StatusCode),
    #[error("HTTP connection error: {0}")]
    HttpConnection(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("internal error: {0}")]
    JoinError(
        #[from]
        #[source]
        tokio::task::JoinError,
    ),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct HttpConnectConnector {
    proxy_uri: http::Uri,
}

impl HttpConnectConnector {
    pub fn new(proxy_uri: http::Uri) -> Self {
        Self { proxy_uri }
    }

    async fn call_async(&self, dst: http::Uri) -> Result<HttpConnectStream> {
        let proxy_addr = (
            self.proxy_uri
                .host()
                .ok_or_else(|| Error::InvalidProxyUri(self.proxy_uri.clone()))?,
            self.proxy_uri
                .port_u16()
                .ok_or_else(|| Error::InvalidProxyUri(self.proxy_uri.clone()))?,
        );

        let stream = TcpStream::connect(proxy_addr).await?;

        let (mut request_sender, connection) = conn::handshake(stream).await?;

        // spawn a task to poll the connection and drive the HTTP state
        let task = tokio::spawn(async move {
            let parts = connection.without_shutdown().await;
            match parts {
                Ok(parts) => Ok((parts.io, parts.read_buf)),
                Err(cause) => Err(Error::HttpConnection(cause)),
            }
        });

        let target = format!(
            "{}:{}",
            dst.host()
                .ok_or_else(|| Error::InvalidConnectUri(self.proxy_uri.clone()))?,
            dst.port_u16()
                .ok_or_else(|| Error::InvalidConnectUri(self.proxy_uri.clone()))?,
        );
        let request = Request::connect(&target)
            .header(HOST, target)
            .body(Body::from(""))?;

        let response = request_sender.send_request(request).await?;

        let (io, buf) = if response.status() == StatusCode::OK {
            let _body = hyper::body::to_bytes(response.into_body()).await?;
            request_sender.ready().await?;
            let (io, read_buf) = task.await??;
            (io, read_buf)
        } else {
            return Err(Error::UnexpectedHttpStatusCode(response.status()));
        };

        let (rd, wr) = tokio::io::split(io);

        let buf = Cursor::new(buf);

        let rd = <Cursor<Bytes> as AsyncReadExt>::chain(buf, rd);
        let rd = Box::new(rd);
        let stream = HttpConnectStream::new(rd, wr);

        Ok(stream)
    }
}

impl hyper::service::Service<http::Uri> for HttpConnectConnector {
    type Response = HttpConnectStream;

    type Error = Error;

    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: http::Uri) -> Self::Future {
        let this = self.clone();

        let fut = async move { this.call_async(dst).await };

        Box::pin(fut)
    }
}
