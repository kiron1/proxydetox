#![allow(clippy::type_complexity)]

use super::http_proxy_stream::HttpProxyStream;
use detox_net::HostAndPort;
use http::Uri;
use hyper::service::Service;
use paclib::Proxy;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};
use tokio::net::TcpStream;
use tokio_native_tls::{native_tls, TlsConnector};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("error connecting to {0}")]
    ConnectError(HostAndPort, #[source] std::io::Error),
    #[error("TLS error")]
    TlsError(
        #[from]
        #[source]
        tokio_native_tls::native_tls::Error,
    ),
}

#[derive(Clone, Debug)]
pub struct HttpProxyConnector {
    proxy: Proxy,
    tls: TlsConnector,
}

impl HttpProxyConnector {
    pub fn new(proxy: Proxy) -> Self {
        let tls = native_tls::TlsConnector::new()
            .map(Into::into)
            .unwrap_or_else(|e| panic!("HttpProxyConnector::new() failure: {}", e));
        Self { proxy, tls }
    }

    async fn call_async(&mut self, _dst: Uri) -> std::result::Result<HttpProxyStream, Error> {
        let stream = TcpStream::connect(self.proxy.endpoint().to_pair())
            .await
            .map_err(|e| Error::ConnectError(self.proxy.endpoint().clone(), e))?;
        let local_addr = stream.local_addr().ok();
        let remote_addr = stream.peer_addr().ok();
        let stream = match self.proxy {
            Proxy::Http(_) => stream.into(),
            Proxy::Https(_) => {
                let tls = self.tls.connect(self.proxy.host(), stream).await?;
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
