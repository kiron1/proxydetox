#![allow(clippy::type_complexity)]

use super::http_proxy_stream::HttpProxyStream;
use http::Uri;
use hyper::service::Service;
use std::{
    future::Future,
    pin::Pin,
    task::{self, Poll},
};
use tokio::net::TcpStream;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("lookup error")]
    LookupError,
    #[error("connect error: {0}")]
    ConnectError(#[from] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct HttpProxyConnector {
    host: String,
    port: u16,
}

impl HttpProxyConnector {
    pub fn new((host, port): (String, u16)) -> Self {
        Self { host, port }
    }

    async fn call_async(&mut self, _dst: Uri) -> std::result::Result<HttpProxyStream, Error> {
        let stream = TcpStream::connect((self.host.clone(), self.port))
            .await
            .map_err(Error::ConnectError)?;

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
