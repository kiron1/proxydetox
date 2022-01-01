pub mod auth;
pub mod client;
pub mod detox;
pub mod io;
pub mod net;

use crate::net::original_destination_address;
pub use net::http_file;
use parking_lot::Mutex;
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

#[derive(Debug)]
pub enum Command {
    Restart,
    Shutdown,
}

pub struct Server {
    port: u16,
    tx: tokio::sync::mpsc::Sender<Command>,
    rx: Arc<Mutex<tokio::sync::mpsc::Receiver<Command>>>,
}

impl Server {
    pub fn new(port: u16) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<Command>(32);

        Self {
            port,
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn control_channel(&self) -> tokio::sync::mpsc::Sender<Command> {
        self.tx.clone()
    }

    pub async fn run(
        &mut self,
        session: detox::Session,
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use futures::{
            future::FutureExt, // for `.fuse()`
            pin_mut,
            select,
        };

        let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
        let cmd_rx = self.rx.clone();
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));

        loop {
            let server = hyper::Server::bind(&addr).serve(detox::Service::new(session.clone()));
            let server = server
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.recv().await;
                })
                .fuse();

            tracing::info!("Listening on http://{}", addr);

            let mut cmd_rx = cmd_rx.lock();
            let recv = cmd_rx.recv().fuse();

            pin_mut!(server, recv);

            select! {
                server_result = server => {
                    if let Err(e) = server_result {
                        tracing::error!("server error: {}", e);
                        return Err(e.into());
                    } else {
                        tracing::info!("shuting down");
                        break;
                    }
                },
                cmd = recv => {
                    match cmd {
                        Some(Command::Shutdown) => { let _ = shutdown_tx.send(()).await;},
                        Some(Command::Restart) => continue,
                        None => break,
                    }
                },
            }
        }
        Ok(())
    }
}

pub async fn run_transparent(
    port: u16,
    session: detox::Session,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let listener = TcpListener::bind(addr).await?;
    tracing::info!("Listening on http://{}", addr);

    while let Ok((stream, _addr)) = listener.accept().await {
        let dst_addr = original_destination_address(&stream);
        let mut session = session.clone();

        tokio::spawn(async move {
            let _ = session.transparent(stream, dst_addr).await;
        });
    }

    Ok(())
}
