mod httpd;

use std::{io::Cursor, net::SocketAddr};

use hyper::Body;
use tokio::{net::TcpStream, sync::oneshot, task};

use proxydetox::auth::{netrc, AuthenticatorFactory};

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

    // pub(crate) fn proxy_uri(&self) -> http::Uri {
    //     http::Uri::builder()
    //         .scheme("http")
    //         .authority(self.server_control.local_addr().to_string())
    //         .build()
    //         .unwrap()
    // }

    pub(crate) async fn send(&self, request: http::Request<Body>) -> http::Response<Body> {
        let stream = TcpStream::connect(self.proxy_addr()).await.unwrap();
        let (mut request_sender, connection) =
            hyper::client::conn::handshake(stream).await.unwrap();

        // spawn a task to poll the connection and drive the HTTP state
        let _task = tokio::spawn(async move {
            connection.await.ok();
        });

        let response = request_sender.send_request(request).await.unwrap();
        response
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
    pub(crate) fn build(self) -> Environment {
        let auth = self
            .netrc_content
            .map(|x| netrc::Store::new(Cursor::new(x)).unwrap())
            .map(AuthenticatorFactory::basic);

        let session = proxydetox::Session::builder()
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
