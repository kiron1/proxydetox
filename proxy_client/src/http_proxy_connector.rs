#![allow(clippy::type_complexity)]

use super::http_proxy_stream::HttpProxyStream;
use detox_net::{HostAndPort, TcpKeepAlive};
use http::Uri;
use hyper::service::Service;
use paclib::Proxy;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error connecting to {0}")]
    ConnectError(HostAndPort, #[source] std::io::Error),
    #[error("TLS error")]
    TlsError(
        #[from]
        #[source]
        std::io::Error,
    ),
}

#[derive(Clone)]
pub struct HttpProxyConnector {
    proxy: Proxy,
    tls: TlsConnector,
    tcp_keepalive: TcpKeepAlive,
}

impl HttpProxyConnector {
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

    async fn call_async(&mut self, _dst: Uri) -> std::result::Result<HttpProxyStream, Error> {
        let stream = TcpStream::connect(self.proxy.endpoint().to_pair())
            .await
            .map_err(|e| Error::ConnectError(self.proxy.endpoint().clone(), e))?;
        self.tcp_keepalive.apply(&stream).ok();
        let local_addr = stream.local_addr().ok();
        let remote_addr = stream.peer_addr().ok();
        let stream = match self.proxy {
            Proxy::Http(_) => stream.into(),
            Proxy::Https(_) => {
                let domain = rustls::ServerName::try_from(self.proxy.host()).map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid domain name")
                })?;
                let tls = self.tls.connect(domain, stream).await?;
                tls.into()
            }
        };

        let stream = HttpProxyStream::new(stream);
        let stream = stream.with_addr(local_addr, remote_addr);

        Ok(stream)
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

impl std::fmt::Debug for HttpProxyConnector {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("HttpProxyConnector")
            .field("proxy", &self.proxy)
            .field("tcp_keepalive", &self.tcp_keepalive)
            .finish()
    }
}
