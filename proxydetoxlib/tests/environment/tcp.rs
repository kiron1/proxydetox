use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

pub struct Server {
    spawn_handle: JoinHandle<()>,
    shutdown_token: CancellationToken,
    local_addr: SocketAddr,
}

impl Server {
    pub(crate) async fn new<Ret>(handler: impl Fn(TcpStream) -> Ret + Send + Sync + 'static) -> Self
    where
        Ret: Future + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .unwrap();
        let local_addr = listener.local_addr().unwrap();
        let shutdown_token = CancellationToken::new();

        let spawn_handle = tokio::spawn({
            let handler = handler.clone();
            let shutdown_token = shutdown_token.clone();

            async move {
                while !shutdown_token.is_cancelled() {
                    tokio::select! {
                        stream = listener.accept() =>  {
                            let (stream, _addr) = stream.unwrap();
                            tokio::spawn({
                                let handler = handler.clone();
                                async move {
                                    handler(stream).await;

                            }});
                        },
                        _ = shutdown_token.cancelled() => {
                            tracing::debug!("got shutdown rx");
                        },
                    };
                }
            }
        });

        Self {
            spawn_handle,
            shutdown_token,
            local_addr,
        }
    }

    pub(crate) fn uri(&self) -> http::Uri {
        http::Uri::builder()
            .scheme("tcp")
            .authority(self.origin())
            .path_and_query("/")
            .build()
            .unwrap()
    }

    pub(crate) fn origin(&self) -> String {
        self.local_addr.to_string()
    }

    pub(crate) async fn shutdown(self) {
        tracing::trace!("shutdown");
        self.shutdown_token.cancel();
        self.spawn_handle.await.ok();
    }
}
