use super::http_proxy_stream::HttpProxyStream;
use http::Uri;
use hyper::service::Service;
use std::fmt::{Error, Formatter};
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};
use tokio::net::TcpStream;

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

#[derive(Clone, Debug)]
pub struct HttpProxyConnector {
    proxy_uri: Uri,
}

impl HttpProxyConnector {
    pub fn new(proxy_uri: Uri) -> Self {
        Self { proxy_uri }
    }

    async fn call_async(
        &mut self,
        _: Uri,
    ) -> std::result::Result<HttpProxyStream, HttpProxyConnectorError> {
        let port = self.proxy_uri.port_u16().unwrap_or(3128);
        let host = self
            .proxy_uri
            .host()
            .ok_or(HttpProxyConnectorError::LookupError)?;

        let stream = TcpStream::connect((host, port))
            .await
            .map_err(HttpProxyConnectorError::ConnectError)?;

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
