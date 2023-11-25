use http::Uri;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct Server {
    spawn_handle: JoinHandle<()>,
    shutdown_token: CancellationToken,
    local_addr: SocketAddr,
}

impl Server {
    pub(crate) async fn new(
        handler: impl Fn(Request<hyper::body::Incoming>) -> Response<crate::environment::Body>
            + Send
            + Sync
            + 'static,
    ) -> Self {
        let handler = Arc::new(handler);
        let service = service_fn(move |r| {
            let handler = handler.clone();
            async move { Ok::<_, Infallible>(handler(r)) }
        });
        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = tokio::net::TcpListener::bind(addr).await.expect("bind");
        let local_addr = listener.local_addr().expect("local_addr");

        let shutdown_token = CancellationToken::new();

        let spawn_handle = tokio::spawn({
            let shutdown_token = shutdown_token.clone();
            async move {
                loop {
                    tokio::select! {
                        stream = listener.accept() => {
                            let (stream, _addr) = stream.expect("accept");
                            let stream = TokioIo::new(stream);
                            let service = service.clone();
                            tokio::task::spawn({
                                async move {
                                    let conn = hyper::server::conn::http1::Builder::new()
                                        .serve_connection(stream, service)
                                        .with_upgrades()
                                        .await;
                                    if let Err(err) = conn
                                    {
                                        tracing::error!("Failed to serve connection: {:?}", err);
                                    }
                                }
                            }
                        );
                        }
                        _ = shutdown_token.cancelled() => {
                            break;
                        }
                    }
                }
            }
        });

        Server {
            local_addr,
            spawn_handle,
            shutdown_token,
        }
    }

    pub(crate) fn host_and_port(&self) -> String {
        self.local_addr.to_string()
    }

    pub(crate) fn uri(&self) -> http::uri::Builder {
        Uri::builder()
            .scheme("http")
            .authority(self.local_addr.to_string())
            .path_and_query("/")
    }

    pub(crate) async fn shutdown(self) {
        self.shutdown_token.cancel();
        self.spawn_handle.await.unwrap();
    }
}
