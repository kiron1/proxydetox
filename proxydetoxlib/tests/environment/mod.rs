#![allow(dead_code)]
mod httpd;
pub mod tcp;

use std::{io::Cursor, net::SocketAddr};

use hyper::Body;
use tokio::{net::TcpStream, sync::oneshot, task};

use proxydetoxlib::auth::{netrc, AuthenticatorFactory};

pub use httpd::Server;

pub(crate) struct Environment {
    server_handle: task::JoinHandle<()>,
    shutdown_tx: oneshot::Sender<()>,
    local_addr: SocketAddr,
}

impl Environment {
    pub(crate) fn new() -> Self {
        Self::builder().build()
    }

    pub(crate) fn builder() -> Builder {
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .pretty()
            .with_timer(tracing_subscriber::fmt::time::uptime())
            .try_init()
            .ok();
        Default::default()
    }

    pub(crate) fn proxy_addr(&self) -> SocketAddr {
        self.local_addr
    }

    pub(crate) fn proxy_uri(&self) -> http::uri::Builder {
        http::Uri::builder()
            .scheme("http")
            .authority(self.local_addr.to_string())
    }

    pub(crate) async fn send(&self, request: http::Request<Body>) -> http::Response<Body> {
        let stream = TcpStream::connect(self.proxy_addr()).await.unwrap();
        let (mut request_sender, connection) =
            hyper::client::conn::handshake(stream).await.unwrap();

        // spawn a task to poll the connection and drive the HTTP state
        let _task = tokio::spawn(async move {
            connection.await.ok();
        });

        request_sender.send_request(request).await.unwrap()
    }

    pub(crate) async fn connect(
        &self,
        request: http::Request<Body>,
    ) -> (http::Response<Body>, hyper::body::Bytes, TcpStream) {
        let stream = TcpStream::connect(self.proxy_addr()).await.unwrap();
        stream.set_nodelay(true).unwrap();
        let (mut request_sender, connection) =
            hyper::client::conn::handshake(stream).await.unwrap();

        // spawn a task to poll the connection and drive the HTTP state
        let parts = tokio::spawn(async move { connection.without_shutdown().await.unwrap() });

        let response = request_sender.send_request(request).await.unwrap();
        let parts = parts.await.unwrap();
        (response, parts.read_buf, parts.io)
    }

    pub(crate) async fn shutdown(self) {
        self.shutdown_tx.send(()).unwrap();
        self.server_handle.await.ok();
    }
}

#[derive(Debug, Default)]
pub(crate) struct Builder {
    pac_script: Option<String>,
    netrc_content: Option<String>,
    always_use_connect: bool,
}

impl Builder {
    pub(crate) fn pac_script(mut self, pac_script: Option<String>) -> Self {
        self.pac_script = pac_script;
        self
    }

    pub(crate) fn netrc_content(mut self, netrc_content: Option<String>) -> Self {
        self.netrc_content = netrc_content;
        self
    }

    pub(crate) fn build(self) -> Environment {
        let auth = self
            .netrc_content
            .map(|x| netrc::Store::new(Cursor::new(x)).unwrap())
            .map(AuthenticatorFactory::basic);

        let session = proxydetoxlib::Session::builder()
            .pac_script(self.pac_script)
            .authenticator_factory(auth)
            .always_use_connect(self.always_use_connect)
            .build();

        let local_addr = SocketAddr::from(([127, 0, 0, 1], 0));
        let server = hyper::Server::bind(&local_addr).serve(session);
        let local_addr = server.local_addr();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let server = server.with_graceful_shutdown(async {
            shutdown_rx.await.ok();
        });

        let server_handle = tokio::spawn(async { server.await.unwrap() });

        Environment {
            server_handle,
            shutdown_tx,
            local_addr,
        }
    }
}
