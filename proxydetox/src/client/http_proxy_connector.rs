use super::http_proxy_stream::HttpProxyStream;
use hyper::{service::Service, Uri};
use std::fmt::{Error, Formatter};
use std::{
    future::Future,
    net::{IpAddr, SocketAddr},
    pin::Pin,
    task::{self, Poll},
};
use tokio::net::TcpStream;
use url::Url;

#[derive(Debug)]
pub enum HttpProxyConnectorError {
    LookupError,
    ConnectError(std::io::Error),
}

impl std::error::Error for HttpProxyConnectorError {}

impl std::fmt::Display for HttpProxyConnectorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            HttpProxyConnectorError::LookupError => write!(f, "lookup error"),
            HttpProxyConnectorError::ConnectError(ref err) => write!(f, "connect error: {}", err),
        }
    }
}

#[derive(Clone)]
pub struct HttpProxyConnector {
    proxy_url: Url,
}

impl HttpProxyConnector {
    pub fn new(proxy_url: Url) -> Self {
        Self { proxy_url }
    }

    async fn call_async(
        &mut self,
        _: Uri,
    ) -> std::result::Result<HttpProxyStream, HttpProxyConnectorError> {
        let port = self.proxy_url.port().unwrap_or(3128);
        let addr = match self.proxy_url.host() {
            None => Err(HttpProxyConnectorError::LookupError)?,
            Some(url::Host::Domain(host)) => {
                let mut addr = tokio::net::lookup_host(format!("{}:{}", host, port))
                    .await
                    .map_err(|_| HttpProxyConnectorError::LookupError)?;
                addr.next().ok_or(HttpProxyConnectorError::LookupError)?
            }
            Some(url::Host::Ipv4(addr)) => SocketAddr::new(IpAddr::V4(addr), port),
            Some(url::Host::Ipv6(addr)) => SocketAddr::new(IpAddr::V6(addr), port),
        };
        let stream = TcpStream::connect(addr)
            .await
            .map_err(|e| HttpProxyConnectorError::ConnectError(e))?;
        Ok(HttpProxyStream::new(stream))
    }
}

impl Service<Uri> for HttpProxyConnector {
    type Response = HttpProxyStream;
    type Error = HttpProxyConnectorError;
    // We can't "name" an `async` generated future.
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut task::Context<'_>,
    ) -> Poll<Result<(), HttpProxyConnectorError>> {
        // This connector is always ready.
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let mut self_ = self.clone();
        Box::pin(async move { self_.call_async(dst).await })
    }
}
