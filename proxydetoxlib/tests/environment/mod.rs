#![allow(dead_code)]
pub mod httpd;
pub mod tcp;

use std::{
    io::{Cursor, Read},
    net::SocketAddr,
};

use bytes::Buf;
use http::{header::CONNECTION, HeaderMap, HeaderValue, StatusCode};
use http_body_util::BodyExt;
use hyper_util::rt::TokioIo;
use tokio::{
    net::{TcpListener, TcpStream},
    task,
};

use detox_auth::{netrc, AuthenticatorFactory};

use tokio_stream::wrappers::TcpListenerStream;
use tokio_util::sync::CancellationToken;

pub type Body = http_body_util::combinators::BoxBody<bytes::Bytes, hyper::Error>;

pub(crate) struct Environment {
    server_handle: task::JoinHandle<()>,
    shutdown_token: CancellationToken,
    local_addr: SocketAddr,
}

impl Environment {
    pub(crate) async fn new() -> Self {
        Self::builder().build().await
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

    pub(crate) async fn send(
        &self,
        mut request: http::Request<Body>,
    ) -> http::Response<hyper::body::Incoming> {
        let stream = TcpStream::connect(self.proxy_addr()).await.unwrap();
        let (mut request_sender, connection) =
            hyper::client::conn::http1::handshake(TokioIo::new(stream))
                .await
                .unwrap();

        let h = request
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
        assert!(h.is_none());

        let (response, connection) = tokio::join!(request_sender.send_request(request), connection);
        let _ = connection.unwrap();
        response.unwrap()
    }

    pub(crate) async fn connect(
        &self,
        mut request: http::Request<Body>,
    ) -> (
        StatusCode,
        HeaderMap,
        hyper::Result<hyper::upgrade::Upgraded>,
    ) {
        request
            .headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));
        let stream = TcpStream::connect(self.proxy_addr()).await.unwrap();
        let (mut request_sender, connection) =
            hyper::client::conn::http1::handshake(TokioIo::new(stream))
                .await
                .unwrap();

        let send_request = async move {
            let response = request_sender.send_request(request).await.unwrap();
            let status = response.status();
            let headers = response.headers().clone();
            let upgraded = hyper::upgrade::on(response).await;
            (status, headers, upgraded)
        };
        let (response, _connection) = tokio::join!(send_request, connection.with_upgrades());

        response
    }

    pub(crate) async fn shutdown(self) {
        self.shutdown_token.cancel();
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

    pub(crate) async fn build(self) -> Environment {
        let auth = self
            .netrc_content
            .map(|n| netrc::Store::new(Cursor::new(n)).unwrap())
            .map(AuthenticatorFactory::basic);

        let context = proxydetoxlib::Context::builder()
            .pac_script(
                self.pac_script
                    .unwrap_or_else(|| proxydetoxlib::DEFAULT_PAC_SCRIPT.to_string()),
            )
            .authenticator_factory(auth)
            .always_use_connect(self.always_use_connect)
            .build();

        let listener = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
            .await
            .expect("bind");
        let local_addr = listener.local_addr().expect("local_addr");
        let listener = TcpListenerStream::new(listener);
        let shutdown_token = CancellationToken::new();

        let server_handle = tokio::spawn({
            let shutdown_token = shutdown_token.clone();
            async move {
                let (server, control) = proxydetoxlib::server::Proxy::new(listener, context);

                let mut server = std::pin::pin!(server);
                tokio::select! {
                    s = server.run() => {
                        s.unwrap();
                    },
                    _ = shutdown_token.cancelled() => {
                        control.shutdown();
                    }
                }
            }
        });

        Environment {
            server_handle,
            shutdown_token,
            local_addr,
        }
    }
}

pub(crate) fn empty() -> Body {
    http_body_util::Empty::new()
        .map_err(|never| match never {})
        .boxed()
}

pub(crate) fn full<T: Into<bytes::Bytes>>(chunk: T) -> Body {
    http_body_util::Full::new(chunk.into())
        .map_err(|never| match never {})
        .boxed()
}

pub(crate) async fn read_to_string(body: hyper::body::Incoming) -> String {
    let body = body.collect().await.expect("receive body").aggregate();
    let mut data = Vec::new();
    body.reader().read_to_end(&mut data).expect("read_to_end");
    String::from_utf8(data).expect("UTF-8 data")
}
