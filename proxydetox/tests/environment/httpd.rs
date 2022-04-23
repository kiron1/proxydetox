use http::Uri;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response};
use std::sync::Arc;
use std::{convert::Infallible, net::SocketAddr};
use tokio::task::JoinHandle;

pub struct Server {
    spawn_handle: JoinHandle<()>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    local_addr: SocketAddr,
}

impl Server {
    pub(crate) fn new(
        handler: impl Fn(Request<Body>) -> Response<Body> + Send + Sync + 'static,
    ) -> Self {
        let handler = Arc::new(handler);
        let make_svc = make_service_fn(move |_conn| {
            let handler = handler.clone();
            async move {
                let handler = handler.clone();
                Ok::<_, Infallible>(service_fn(move |req| {
                    let handler = handler.clone();
                    async move { Ok::<_, Infallible>(handler(req)) }
                }))
            }
        });

        let addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = hyper::Server::bind(&addr).serve(make_svc);
        let local_addr = server.local_addr();

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
        let server = server.with_graceful_shutdown(async move {
            shutdown_rx.await.ok();
        });

        let spawn_handle = tokio::spawn(async move {
            server.await.unwrap();
        });

        Server {
            local_addr,
            spawn_handle,
            shutdown_tx,
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
        self.shutdown_tx.send(()).ok();
        self.spawn_handle.await.unwrap();
    }
}
