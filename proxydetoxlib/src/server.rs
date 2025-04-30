use std::{net::SocketAddr, sync::Arc};

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

pub struct Proxy;

#[derive(Clone)]
pub struct Server<A> {
    acceptor: A,
    http_server: http1::Builder,
    context: Arc<Context>,
    shutdown_request: CancellationToken,
    shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
}

pub struct Control {
    shutdown_request: CancellationToken,
    shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
    shutdown_complete_rx: tokio::sync::mpsc::Receiver<()>,
}

struct HttpHandler {
    addr: SocketAddr,
    conn: hyper::server::conn::http1::Connection<TokioIo<TcpStream>, Session>,
    shutdown_request: CancellationToken,
    shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
}

impl HttpHandler {
    #[instrument(skip(self), fields(peer = debug(self.addr)))]
    async fn run(self) {
        let HttpHandler {
            addr: _,
            conn,
            shutdown_request,
            shutdown_complete_tx,
        } = self;
        let conn = conn.with_upgrades();
        tracing::debug!("http peer connected");
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
                    tracing::debug!("shutdown requested");
                    conn.as_mut().graceful_shutdown();
                }
            }
        }
        drop(shutdown_complete_tx);
    }
}

struct TcpHandler {
    addr: SocketAddr,
    dst: SocketAddr,
    context: Arc<Context>,
    shutdown_request: CancellationToken,
    shutdown_complete_tx: tokio::sync::mpsc::Sender<()>,
}

impl TcpHandler {
    #[instrument(skip(self), fields(peer = debug(self.addr)))]
    async fn run(self) {
        let TcpHandler {
            addr: _,
            dst,
            context,
            shutdown_request,
            shutdown_complete_tx,
        } = self;
        tracing::debug!("tcp peer connected");
        let uri = Uri::builder()
            .scheme(http::uri::Scheme::HTTP)
            .authority(dst.to_string().parse().expect("IP is valid authority"))
            .build()
            .expect("URI");
        let proxies = context.find_proxy(uri).await;
        let conn = proxies.clone().into_iter().map({
            let cx = self.context.clone();
            let method = req.method();
            let uri = req.uri();
            move |p| {
                let cx = cx.clone();
                let race = cx.race_connect;
                async move {
                    let r = cx.connect(p, method.clone(), uri.clone()).await;
                    if let Err(ref cause) = r {
                        if race {
                            tracing::debug!(%cause, "unable to connect");
                        } else {
                            tracing::warn!(%cause, "unable to connect");
                        }
                    }
                    r
                }
            }
        });
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
                    tracing::debug!("shutdown requested");
                    conn.as_mut().graceful_shutdown();
                }
            }
        }
        drop(shutdown_complete_tx);
    }
}

impl Proxy {
    #[allow(clippy::new_ret_no_self)]
    pub fn new<A>(acceptor: A, context: Arc<Context>) -> (Server<A>, Control)
    where
        A: futures_util::Stream<Item = std::io::Result<TcpStream>>,
    {
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
        };
        let control = Control {
            shutdown_request,
            shutdown_complete_rx,
            shutdown_complete_tx,
        };
        (server, control)
    }
}

impl<A> Server<A>
where
    A: futures_util::Stream<Item = std::io::Result<tokio::net::TcpStream>> + Send + Unpin + 'static,
{
    fn accept(&mut self, stream: std::io::Result<tokio::net::TcpStream>) -> std::io::Result<()> {
        let stream = match stream {
            Ok(stream) => stream,
            Err(cause) => {
                tracing::error!(%cause, "listener error");
                return Err(cause);
            }
        };
        let addr = stream.peer_addr().expect("peer_addr");
        let orig_dst_addr = crate::socket::original_destination_address(&stream);
        if let Some(dst) = orig_dst_addr {
            let handler = TcpHandler {
                addr,
                dst,
                context: self.context.clone(),
                shutdown_request: self.shutdown_request.clone(),
                shutdown_complete_tx: self.shutdown_complete_tx.clone(),
            };
            tokio::spawn(handler.run());
        } else {
            let conn = self.http_server.serve_connection(
                TokioIo::new(stream),
                Session::new(self.context.clone(), addr, orig_dst_addr),
            );
            let handler = HttpHandler {
                addr,
                conn,
                shutdown_request: self.shutdown_request.clone(),
                shutdown_complete_tx: self.shutdown_complete_tx.clone(),
            };
            tokio::spawn(handler.run());
        }
        Ok(())
    }
    #[instrument(skip(self))]
    pub async fn run(&mut self) -> std::io::Result<()> {
        while !self.shutdown_request.is_cancelled() {
            tokio::select! {
                _ = self.shutdown_request.cancelled() => {
                    break;
                },
                stream = self.acceptor.next() => {
                    self.accept(stream.expect("infinite stream of TcpStream"))?
                },
            }
        }

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

    pub async fn wait_with_timeout(
        self,
        timeout: std::time::Duration,
    ) -> std::result::Result<(), WaitError> {
        let Self {
            mut shutdown_complete_rx,
            shutdown_request: _,
            shutdown_complete_tx,
        } = self;
        drop(shutdown_complete_tx);
        select! {
            _ = shutdown_complete_rx.recv() => {},
            _ = tokio::time::sleep(timeout) => {
                return Err(WaitError::TimeoutExpired);
            }
        }
        Ok(())
    }
}
