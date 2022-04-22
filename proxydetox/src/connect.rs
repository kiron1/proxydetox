use http::{Request, Response, Uri};
use hyper::Body;
use std::{future::Future, pin::Pin};
use tokio::{io::copy_bidirectional, net::TcpStream};
use tracing_futures::Instrument;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("stream already taken")]
    StreamAlreadyTaken,
}

/// A `tower::Service` which establishes TCP connection.
///
/// The Response from this service is a service which can be used to upgrade a http::Request to
/// establish a connnected stream.
#[derive(Debug, Default)]
pub struct Connect;

impl Connect {
    pub fn new() -> Self {
        Self::default()
    }
}

impl tower::Service<Uri> for Connect {
    type Response = Handshake;
    type Error = tokio::io::Error;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let res = async move {
            match (dst.host(), dst.port_u16()) {
                (Some(host), Some(port)) => {
                    TcpStream::connect((host, port)).await.map(Handshake::new)
                }
                (_, _) => Err(tokio::io::Error::new(
                    tokio::io::ErrorKind::AddrNotAvailable,
                    "invalid URI",
                )),
            }
        };
        Box::pin(res)
    }
}

#[derive(Debug)]
pub struct Handshake {
    stream: Option<TcpStream>,
}

impl Handshake {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Some(stream),
        }
    }
}

impl tower::Service<Request<Body>> for Handshake {
    type Response = Response<Body>;
    type Error = Error;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let stream = self.stream.take();
        let res = async move {
            if let Some(mut stream) = stream {
                let remote_addr = stream.peer_addr().ok();
                let upgrade_task = async move {
                    match hyper::upgrade::on(req).await {
                        Ok(mut upgraded) => {
                            let cp = copy_bidirectional(&mut upgraded, &mut stream).await;
                            if let Err(cause) = cp {
                                tracing::error!(%cause, "tunnel error")
                            }
                        }
                        Err(cause) => tracing::error!(%cause, "upgrade error"),
                    }
                };
                tokio::task::spawn(
                    upgrade_task.instrument(tracing::info_span!("upgrade connect", ?remote_addr)),
                );
                Ok(Response::new(Body::empty()))
            } else {
                tracing::error!("stream already taken");
                Err(Error::StreamAlreadyTaken)
            }
        };
        let res = res.instrument(tracing::trace_span!("Handshake::call"));
        Box::pin(res)
    }
}
