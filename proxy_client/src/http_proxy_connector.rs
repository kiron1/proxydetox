#![allow(clippy::type_complexity)]

use super::http_proxy_stream::HttpProxyStream;
use detox_net::HostAndPort;
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
    #[error("error connecting to {0}")]
    ConnectError(HostAndPort, #[source] std::io::Error),
}

#[derive(Clone, Debug)]
pub struct HttpProxyConnector {
    endpoint: HostAndPort,
}

impl HttpProxyConnector {
    pub fn new(endpoint: HostAndPort) -> Self {
        Self { endpoint }
    }

    async fn call_async(&mut self, _dst: Uri) -> std::result::Result<HttpProxyStream, Error> {
        let stream = TcpStream::connect(self.endpoint.to_pair())
            .await
            .map_err(move |e| Error::ConnectError(self.endpoint.clone(), e))?;

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
