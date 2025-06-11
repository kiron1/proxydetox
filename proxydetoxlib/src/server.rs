use std::{net::SocketAddr, sync::Arc};

use detox_futures::FutureExt;
use futures_util::StreamExt;
use hyper::server::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::{net::TcpStream, select};
use tokio_util::sync::CancellationToken;
use tracing_attributes::instrument;

use crate::{Context, Session};

#[derive(Debug, thiserror::Error)]
pub enum WaitError {
    #[error("timeout expired")]
    TimeoutExpired,
}

pub struct Server<A> {
    acceptor: A,
    http_server: http1::Builder,
    context: Arc<Context>,
    shutdown_request: CancellationToken,
    shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
    shutdown_complete_rx: tokio::sync::mpsc::Receiver<()>,
}

pub struct Control {
    shutdown_request: CancellationToken,
}

pub struct JoinHandle {
    shutdown_complete_rx: tokio::sync::mpsc::Receiver<()>,
}

struct Handler {
    addr: SocketAddr,
    conn: hyper::server::conn::http1::Connection<TokioIo<TcpStream>, Session>,
    shutdown_request: CancellationToken,
    shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
}

impl Handler {
    #[instrument(skip(self), fields(peer = debug(self.addr)))]
    async fn run(self) {
        let Handler {
            addr: _,
            conn,
            shutdown_request,
            shutdown_complete_tx,
        } = self;
        let conn = conn.with_upgrades();
        tracing::debug!("peer connected");
        let mut conn = std::pin::pin!(conn);
        loop {
            select! {
                c = conn.as_mut() => {
                    if let Err(cause) = c {
                        tracing::error!(%cause, "server connection error");
                    }
                    tracing::debug!("peer disconnected");
                    break;
                },
                _ = shutdown_request.cancelled(), if !shutdown_request.is_cancelled() => {
                    tracing::debug!("handler received shutdown requested");
                    conn.as_mut().graceful_shutdown();
                }
            }
        }
        drop(shutdown_complete_tx);
    }
}

impl<A> Server<A>
where
    A: futures_util::Stream<Item = std::io::Result<tokio::net::TcpStream>> + Send + Unpin + 'static,
{
    pub fn new(acceptor: A, context: Arc<Context>) -> (Server<A>, Control) {
        let http_server = {
            let mut b = http1::Builder::new();
            b.preserve_header_case(true);
            b.title_case_headers(true);
            b
        };
        let shutdown_request = CancellationToken::new();
        let (shutdown_complete_tx, shutdown_complete_rx) = tokio::sync::mpsc::channel(1);
        let server = Server::<A> {
            acceptor,
            http_server,
            context,
            shutdown_request: shutdown_request.child_token(),
            shutdown_complete_tx: shutdown_complete_tx.clone(),
            shutdown_complete_rx,
        };
        let control = Control { shutdown_request };
        (server, control)
    }

    #[instrument(skip(self))]
    pub async fn run(self) -> std::io::Result<JoinHandle> {
        let Self {
            mut acceptor,
            http_server,
            context,
            shutdown_request,
            shutdown_complete_tx,
            shutdown_complete_rx,
        } = self;

        while !shutdown_request.is_cancelled() {
            tokio::select! {
                _ = shutdown_request.cancelled() => {
                    break;
                },
                stream = acceptor.next() => {
                    let stream = match stream {
                        Some(Ok(stream))=> {
                            stream
                        },
                        Some(Err(cause)) => {
                            tracing::error!(%cause, "listener error");
                            return Err(cause);
                        },
                        None => unreachable!(),
                    };
                    let addr = stream.peer_addr().expect("peer_addr");
                    let conn = http_server.serve_connection(TokioIo::new(stream), Session::new(context.clone(), addr));
                    let handler = Handler {
                        addr,
                        conn,
                        shutdown_request: shutdown_request.clone(),
                        shutdown_complete_tx: shutdown_complete_tx.clone(),
                    };
                    tokio::spawn(handler.run());
                },
            }
        }

        drop(acceptor);

        Ok(JoinHandle {
            shutdown_complete_rx,
        })
    }
}

impl JoinHandle {
    pub async fn wait_with_timeout(
        mut self,
        timeout: std::time::Duration,
    ) -> std::result::Result<(), WaitError> {
        self.shutdown_complete_rx
            .recv()
            .timeout(timeout)
            .await
            .map_err(|_| WaitError::TimeoutExpired)?;
        Ok(())
    }
}

impl Control {
    pub fn shutdown(&self) {
        self.shutdown_request.cancel()
    }

    pub fn is_shutdown(&self) -> bool {
        self.shutdown_request.is_cancelled()
    }
}
