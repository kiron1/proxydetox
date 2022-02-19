pub mod auth;
pub mod client;
pub mod detox;
pub mod io;
pub mod net;

use futures::future::join_all;
pub use net::http_file;
use parking_lot::Mutex;

use crate::auth::AuthenticatorFactory;
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

#[derive(Debug)]
pub enum Command {
    Restart,
    Shutdown,
}

pub struct Server {
    pac_script: String,
    auth: AuthenticatorFactory,
    port: u16,
    config: detox::Config,
    tx: tokio::sync::mpsc::Sender<Command>,
    rx: Arc<Mutex<tokio::sync::mpsc::Receiver<Command>>>,
}

impl Server {
    pub fn new(
        pac_script: String,
        auth: AuthenticatorFactory,
        port: u16,
        config: detox::Config,
    ) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel::<Command>(32);

        Self {
            pac_script,
            auth,
            port,
            config,
            tx,
            rx: Arc::new(Mutex::new(rx)),
        }
    }

    pub fn control_channel(&self) -> tokio::sync::mpsc::Sender<Command> {
        self.tx.clone()
    }

    pub async fn run(
        &mut self,
        interfaces: &[IpAddr],
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        use futures::{
            future::FutureExt, // for `.fuse()`
            pin_mut,
            select,
        };

        let (shutdown_tx, _) = tokio::sync::broadcast::channel::<()>(1);
        let cmd_rx = self.rx.clone();

        loop {
            let mut servers = Vec::new();
            for ip in interfaces {
                let mut shutdown_rx = shutdown_tx.subscribe();
                let addr = SocketAddr::new(*ip, self.port);
                let server = hyper::Server::bind(&addr).serve(detox::Service::new(
                    &self.pac_script,
                    self.auth.clone(),
                    self.config.clone(),
                ));
                let server = server.with_graceful_shutdown(async move {
                    let _ = shutdown_rx.recv().await;
                });
                servers.push(server);

                tracing::info!("Listening on http://{}", addr);
            }

            let mut cmd_rx = cmd_rx.lock();
            let recv = cmd_rx.recv().fuse();

            let servers = join_all(servers).fuse();
            pin_mut!(servers, recv);

            select! {
                servers_result = servers => {
                    for server_result in servers_result {
                        if let Err(e) = server_result {
                            tracing::error!("server error: {}", e);
                            return Err(e.into());
                        } else {
                            tracing::info!("shuting down");
                        }
                    }
                            break;
                },
                cmd = recv => {
                    match cmd {
                        Some(Command::Shutdown) => { let _ = shutdown_tx.send(()); },
                        Some(Command::Restart) => continue,
                        None => break,
                    }
                },
            }
        }
        Ok(())
    }
}
