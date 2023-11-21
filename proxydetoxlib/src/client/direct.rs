use std::future::Future;
use std::pin::Pin;

use detox_net::{HostAndPort, TcpKeepAlive};
use http::{Request, Uri};
use tokio::net::TcpStream;

type HyperSendRequest = hyper::client::conn::http1::SendRequest<hyper::body::Body>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid URI: {0}")]
    InvalidUri(
        #[from]
        #[source]
        detox_net::host_and_port::Error,
    ),
    #[error("HTTP error: {0}")]
    Hyper(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("TCP connect to {1} error: {0}")]
    ConnectError(#[source] tokio::io::Error, HostAndPort),
}

#[derive(Default)]
pub struct Direct {
    tcp_keepalive: TcpKeepAlive,
}

impl Direct {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_tcp_keepalive(mut self, keepalive: TcpKeepAlive) -> Self {
        self.tcp_keepalive = keepalive;
        self
    }
}

impl tower::Service<Uri> for Direct {
    type Response = SendRequest;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let tcp_keepalive = self.tcp_keepalive.clone();
        let res = async move {
            let host = HostAndPort::try_from_uri(&dst)?;
            let stream = TcpStream::connect(host.to_pair())
                .await
                .map_err(|e| Error::ConnectError(e, host.clone()))?;
            tcp_keepalive.apply(&stream).ok();
            let (send_request, connection) = hyper::client::conn::http1::handshake(stream).await?;
            tokio::spawn(async move {
                if let Err(cause) = connection.await {
                    tracing::error!(%cause, %host, "error in direct connection");
                }
            });
            Ok(SendRequest(send_request))
        };
        Box::pin(res)
    }
}

pub struct SendRequest(HyperSendRequest);

impl tower::Service<Request<hyper::body::Body>> for SendRequest {
    type Response = hyper::Response<hyper::body::Body>;
    type Error = hyper::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<hyper::body::Body>) -> Self::Future {
        if req.method() != http::Method::CONNECT {
            // strip the authority part of the request URI, since direct clients will only send
            // the path and query in the requst URI part.
            *req.uri_mut() = Uri::builder()
                .path_and_query(
                    req.uri()
                        .path_and_query()
                        .cloned()
                        .unwrap_or_else(|| http::uri::PathAndQuery::from_static("/")),
                )
                .build()
                .expect("request with valid URI expected");
        }
        Box::pin(self.0.send_request(req))
    }
}
