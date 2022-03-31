use std::future::Future;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::task::JoinHandle;

pub struct Server {
    spawn_handle: JoinHandle<()>,
    shutdown_tx: tokio::sync::oneshot::Sender<()>,
    local_addr: SocketAddr,
}

impl Server {
    pub(crate) fn new<Ret>(handler: impl Fn(TcpStream) -> Ret + Send + Sync + 'static) -> Self
    where
        Ret: Future + Send + Sync + 'static,
    {
        let handler = Arc::new(handler);
        let local_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let listener = std::net::TcpListener::bind(&local_addr).unwrap();
        let local_addr = listener.local_addr().unwrap();
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();

        let spawn_handle = tokio::spawn(async move {
            let listener = TcpListener::from_std(listener).unwrap();
            let accept = async move {
                // loop {
                let (stream, _addr) = listener.accept().await.unwrap();

                let handler = Arc::clone(&handler);

                tokio::spawn(async move {
                    stream.set_nodelay(true).unwrap();
                    handler(stream).await;
                });
                tokio::task::yield_now().await;
                // }
            };

            tokio::select! {
                _ = accept =>  {},
                _ = shutdown_rx => {
                    tracing::debug!("got shutdown rx");
                },
            };
        });

        Self {
            spawn_handle,
            shutdown_tx,
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
        self.shutdown_tx.send(()).ok();
        self.spawn_handle.await.ok();
    }
}
