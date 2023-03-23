use crate::HttpConnectStream;
use crate::MaybeTlsStream;
use detox_net::{HostAndPort, TcpKeepAlive};
use http::{header::HOST, Request, StatusCode};
use hyper::{body::Bytes, client::conn, Body};
use paclib::Proxy;
use std::{future::Future, io::Cursor, pin::Pin};
use tokio::{io::AsyncReadExt, net::TcpStream};
use tokio_rustls::TlsConnector;
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
    #[error("error connecting to {0}")]
    ConnectError(HostAndPort, #[source] std::io::Error),
    #[error("TLS error")]
    TlsError(#[source] std::io::Error),
    #[error("internal error: {0}")]
    JoinError(
        #[from]
        #[source]
        tokio::task::JoinError,
    ),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct HttpConnectConnector {
    proxy: Proxy,
    tls: TlsConnector,
    tcp_keepalive: TcpKeepAlive,
}

impl HttpConnectConnector {
    pub fn new(proxy: Proxy, tls: TlsConnector) -> Self {
        Self {
            proxy,
            tls,
            tcp_keepalive: Default::default(),
        }
    }

    pub fn with_tcp_keepalive(mut self, keepalive: TcpKeepAlive) -> Self {
        self.tcp_keepalive = keepalive;
        self
    }

    async fn call_async(&self, dst: http::Uri) -> Result<HttpConnectStream> {
        let stream = TcpStream::connect(self.proxy.endpoint().to_pair())
            .await
            .map_err(|e| Error::ConnectError(self.proxy.endpoint().clone(), e))?;
        self.tcp_keepalive.apply(&stream).ok();
        let local_addr = stream.local_addr().ok();
        let remote_addr = stream.peer_addr().ok();
        let stream: MaybeTlsStream<TcpStream> = match self.proxy {
            Proxy::Http(_) => stream.into(),
            Proxy::Https(_) => {
                let domain = rustls::ServerName::try_from(self.proxy.host()).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid domain name")
                })?;
                let tls = self.tls.connect(domain, stream).await?;
                tls.into()
            }
        };

        let (mut request_sender, connection) = conn::handshake(stream).await?;

        // spawn a task to poll the connection and drive the HTTP state
        let task = tokio::spawn(async move {
            let parts = connection.without_shutdown().await;
            match parts {
                Ok(parts) => Ok((parts.io, parts.read_buf)),
                Err(cause) => Err(Error::HttpConnection(cause)),
            }
        });

        let target = {
            let host = dst
                .host()
                .ok_or_else(|| Error::InvalidConnectUri(dst.clone()))?;
            let port = dst
                .port_u16()
                .ok_or_else(|| Error::InvalidConnectUri(dst.clone()))?;
            format!("{}:{}", host, port)
        };
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
        let wr = Box::new(wr);
        let stream = HttpConnectStream::new(rd, wr);
        let stream = stream.with_addr(local_addr, remote_addr);

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

impl std::fmt::Debug for HttpConnectConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("HttpConnectConnector")
            .field("proxy", &self.proxy)
            .field("tcp_keepalive", &self.tcp_keepalive)
            .finish()
    }
}
