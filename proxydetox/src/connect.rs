use http::{Request, Response, Uri};
use hyper::Body;
use parking_lot::Mutex;
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::{io::copy_bidirectional, net::TcpStream};

/// A `tower::Service` which establishes TCP connection.
///
/// The Response from this service is a service which can be used to upgrade a http::Request to
/// establish a connnected stream.
#[derive(Debug, Default, Clone)]
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

#[derive(Debug, Clone)]
pub struct Handshake {
    stream: Arc<Mutex<Option<TcpStream>>>,
}

impl Handshake {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream: Arc::new(Mutex::new(Some(stream))),
        }
    }
}

impl tower::Service<Request<Body>> for Handshake {
    type Response = Response<Body>;
    type Error = std::convert::Infallible;
    type Future =
        Pin<Box<dyn Future<Output = std::result::Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let stream = {
            let mut gurad = self.stream.lock();
            gurad.take()
        };
        let res = async move {
            tokio::task::spawn(async move {
                match hyper::upgrade::on(req).await {
                    Ok(mut upgraded) => {
                        if let Some(mut stream) = stream {
                            let cp = copy_bidirectional(&mut upgraded, &mut stream).await;
                            if let Err(cause) = cp {
                                tracing::error!(%cause, "tunnel error")
                            }
                        } else {
                            tracing::error!("stream already taken")
                        }
                    }
                    Err(cause) => tracing::error!(%cause, "upgrade error"),
                }
            });

            Ok(Response::new(Body::empty()))
        };
        // let res = res.instrument(tracing::trace_span!("Handshake::call"));
        Box::pin(res)
    }
}
