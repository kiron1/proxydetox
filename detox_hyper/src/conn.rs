use std::{
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};

use bytes::Bytes;
use detox_auth::{Authenticator, AuthenticatorFactory};
use detox_net::{HostAndPort, TcpKeepAlive};
use futures_util::{future::BoxFuture, FutureExt};
use http::{
    header::{CONNECTION, HOST},
    uri::PathAndQuery,
    HeaderValue, Request, Response, Uri,
};
use http_body::Body;
use http_body_util::Empty;
use hyper::{
    body::Incoming as IncomingBody,
    client::conn::http1::{self, Builder},
    upgrade::Upgraded,
};
use hyper_util::rt::TokioIo;
use paclib::Proxy;
use pin_project::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_rustls::{client::TlsStream, TlsConnector};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP protocol error: {0}")]
    Hyper(
        #[from]
        #[source]
        hyper::Error,
    ),
    #[error("HTTP error: {0}")]
    Http(
        #[from]
        #[source]
        http::Error,
    ),
    #[error("invalid header value: {0}")]
    InvalidHeaderValue(
        #[from]
        #[source]
        http::header::InvalidHeaderValue,
    ),
    #[error("Authentication error: {0}")]
    Authentication(
        #[from]
        #[source]
        detox_auth::Error,
    ),
}

#[derive(Debug)]
#[pin_project(project = StreamProj)]
enum AnyStream {
    Http(#[pin] TcpStream),
    Https(#[pin] TlsStream<TcpStream>),
    HttpProxy(#[pin] TcpStream),
    HttpsProxy(#[pin] TlsStream<TcpStream>),
    HttpTunnel(#[pin] TokioIo<Upgraded>),
}

// #[derive(Debug)]
#[pin_project]
pub struct Connection {
    #[pin]
    inner: AnyStream,
    host: Option<String>,
    proxy: Option<HostAndPort>,
    auth: Option<Box<dyn Authenticator>>,
}

pub struct ConnectionBuilder {
    kind: ConnectionKind,
    tcp_keepalive: Option<TcpKeepAlive>,
}

pub enum ConnectionKind {
    Http(HostAndPort),
    Https(HostAndPort, Arc<rustls::ClientConfig>),
    HttpProxy(Proxy, Arc<rustls::ClientConfig>, AuthenticatorFactory),
    HttpTunnel(
        Proxy,
        Arc<rustls::ClientConfig>,
        AuthenticatorFactory,
        HostAndPort,
    ),
}

pub struct SendRequest<B>
where
    B: Body + 'static,
{
    sender: http1::SendRequest<B>,
    conn: http1::Connection<TokioIo<AnyStream>, B>,
    host: Option<String>,
    proxy: Option<HostAndPort>,
    auth: Option<Box<dyn Authenticator>>,
}

impl Connection {
    pub fn http(dst: HostAndPort) -> ConnectionBuilder {
        ConnectionBuilder {
            kind: ConnectionKind::Http(dst),
            tcp_keepalive: Default::default(),
        }
    }

    pub fn https(dst: HostAndPort, tls_config: Arc<rustls::ClientConfig>) -> ConnectionBuilder {
        ConnectionBuilder {
            kind: ConnectionKind::Https(dst, tls_config),
            tcp_keepalive: Default::default(),
        }
    }

    pub fn http_proxy(
        proxy: Proxy,
        tls_config: Arc<rustls::ClientConfig>,
        auth: AuthenticatorFactory,
    ) -> ConnectionBuilder {
        ConnectionBuilder {
            kind: ConnectionKind::HttpProxy(proxy, tls_config, auth),
            tcp_keepalive: Default::default(),
        }
    }

    pub fn http_tunnel(
        proxy: Proxy,
        tls_config: Arc<rustls::ClientConfig>,
        auth: AuthenticatorFactory,
        dst: HostAndPort,
    ) -> ConnectionBuilder {
        ConnectionBuilder {
            kind: ConnectionKind::HttpTunnel(proxy, tls_config, auth, dst),
            tcp_keepalive: Default::default(),
        }
    }

    pub fn is_proxied(&self) -> bool {
        match self.inner {
            AnyStream::Http(_) => false,
            AnyStream::Https(_) => false,
            AnyStream::HttpProxy(_) => true,
            AnyStream::HttpsProxy(_) => true,
            AnyStream::HttpTunnel(_) => false,
        }
    }

    pub async fn handshake<B>(self) -> Result<SendRequest<B>, hyper::Error>
    where
        B: Body + 'static,
        B::Data: Send,
        B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let Connection {
            inner,
            host,
            proxy,
            auth,
        } = self;
        let (sender, conn) = Builder::new()
            .preserve_header_case(true)
            .title_case_headers(true)
            .handshake(TokioIo::new(inner))
            .await?;
        Ok(SendRequest {
            sender,
            conn,
            host,
            proxy,
            auth,
        })
    }

    pub fn proxy(&self) -> &Option<HostAndPort> {
        &self.proxy
    }
}

impl std::fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field("inner", &self.inner)
            .finish()
    }
}

impl ConnectionBuilder {
    pub fn proxy(&self) -> Option<Proxy> {
        match &self.kind {
            ConnectionKind::Http(_) => None,
            ConnectionKind::Https(_, _) => None,
            ConnectionKind::HttpProxy(p, _, _) => Some(p.clone()),
            ConnectionKind::HttpTunnel(p, _, _, _) => Some(p.clone()),
        }
    }
}

impl AsyncWrite for Connection {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<tokio::io::Result<usize>> {
        let this = self.project();
        this.inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<tokio::io::Result<()>> {
        let this = self.project();
        this.inner.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        let this = self.project();
        this.inner.poll_shutdown(cx)
    }
}

impl AsyncRead for Connection {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        let this = self.project();
        this.inner.poll_read(cx, buf)
    }
}

impl ConnectionBuilder {
    pub fn with_tcp_keepalive(self, ka: TcpKeepAlive) -> Self {
        Self {
            kind: self.kind,
            tcp_keepalive: Some(ka),
        }
    }
}

impl std::future::IntoFuture for ConnectionBuilder {
    type Output = std::io::Result<Connection>;

    type IntoFuture = BoxFuture<'static, Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        use ConnectionKind::*;
        match self.kind {
            Http(dst) => async move {
                let stream = TcpStream::connect(dst.to_pair()).await?;
                if let Some(ka) = self.tcp_keepalive {
                    ka.apply(&stream)?;
                }
                Ok(Connection {
                    inner: AnyStream::Http(stream),
                    host: Some(dst.host().to_owned()),
                    proxy: None,
                    auth: None,
                })
            }
            .boxed(),
            Https(dst, tls_config) => async move {
                let stream = TcpStream::connect(dst.to_pair()).await?;
                if let Some(ka) = self.tcp_keepalive {
                    ka.apply(&stream)?;
                }
                let domain = rustls::ServerName::try_from(dst.host()).map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("invalid domain name: {e}"),
                    )
                })?;
                let tls = TlsConnector::from(tls_config);
                let tls = tls.connect(domain, stream).await?;
                Ok(Connection {
                    inner: AnyStream::Https(tls),
                    host: Some(dst.host().to_owned()),
                    proxy: None,
                    auth: None,
                })
            }
            .boxed(),
            HttpProxy(proxy, tls_config, auth) => match proxy {
                Proxy::Http(proxy) => async move {
                    let auth = auth.make(proxy.host()).map_err(|e| {
                        std::io::Error::other(format!(
                            "Unable to build authenticator for '{proxy}': {e}"
                        ))
                    })?;
                    let stream = TcpStream::connect(proxy.to_pair()).await?;
                    if let Some(ka) = self.tcp_keepalive {
                        ka.apply(&stream)?;
                    }
                    Ok(Connection {
                        inner: AnyStream::HttpProxy(stream),
                        host: None,
                        proxy: Some(proxy),
                        auth: Some(auth),
                    })
                }
                .boxed(),
                Proxy::Https(proxy) => async move {
                    let auth = auth.make(proxy.host()).map_err(|e| {
                        std::io::Error::other(format!(
                            "Unable to build authenticator for '{proxy}': {e}"
                        ))
                    })?;
                    let stream = TcpStream::connect(proxy.to_pair()).await?;
                    if let Some(ka) = self.tcp_keepalive {
                        ka.apply(&stream)?;
                    }
                    let domain = rustls::ServerName::try_from(proxy.host()).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("invalid domain name: {e}"),
                        )
                    })?;
                    let tls = TlsConnector::from(tls_config);
                    let tls = tls.connect(domain, stream).await?;
                    Ok(Connection {
                        inner: AnyStream::HttpsProxy(tls),
                        host: None,
                        proxy: Some(proxy),
                        auth: Some(auth),
                    })
                }
                .boxed(),
            },
            HttpTunnel(proxy, tls_config, auth, dst) => match proxy {
                Proxy::Http(proxy) => async move {
                    let stream = TcpStream::connect(proxy.to_pair()).await?;
                    if let Some(ka) = self.tcp_keepalive {
                        ka.apply(&stream)?;
                    }
                    let stream = http_connect(stream, &proxy, &auth, &dst).await?;
                    Ok(Connection {
                        inner: AnyStream::HttpTunnel(TokioIo::new(stream)),
                        host: Some(dst.host().to_owned()),
                        proxy: Some(proxy),
                        auth: None,
                    })
                }
                .boxed(),
                Proxy::Https(proxy) => async move {
                    let stream = TcpStream::connect(proxy.to_pair()).await?;
                    if let Some(ka) = self.tcp_keepalive {
                        ka.apply(&stream)?;
                    }
                    let domain = rustls::ServerName::try_from(proxy.host()).map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Invalid domain name: {e}"),
                        )
                    })?;
                    let tls = TlsConnector::from(tls_config);
                    let stream = tls.connect(domain, stream).await?;
                    let stream = http_connect(stream, &proxy, &auth, &dst).await?;

                    Ok(Connection {
                        inner: AnyStream::HttpTunnel(TokioIo::new(stream)),
                        host: Some(dst.host().to_owned()),
                        proxy: Some(proxy),
                        auth: None,
                    })
                }
                .boxed(),
            },
        }
    }
}

impl AsyncWrite for AnyStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<tokio::io::Result<usize>> {
        match self.project() {
            StreamProj::Http(s) => s.poll_write(cx, buf),
            StreamProj::Https(s) => s.poll_write(cx, buf),
            StreamProj::HttpProxy(s) => s.poll_write(cx, buf),
            StreamProj::HttpsProxy(s) => s.poll_write(cx, buf),
            StreamProj::HttpTunnel(s) => s.poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<tokio::io::Result<()>> {
        match self.project() {
            StreamProj::Http(s) => s.poll_flush(cx),
            StreamProj::Https(s) => s.poll_flush(cx),
            StreamProj::HttpProxy(s) => s.poll_flush(cx),
            StreamProj::HttpsProxy(s) => s.poll_flush(cx),
            StreamProj::HttpTunnel(s) => s.poll_flush(cx),
        }
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        match self.project() {
            StreamProj::Http(s) => s.poll_shutdown(cx),
            StreamProj::Https(s) => s.poll_shutdown(cx),
            StreamProj::HttpProxy(s) => s.poll_shutdown(cx),
            StreamProj::HttpsProxy(s) => s.poll_shutdown(cx),
            StreamProj::HttpTunnel(s) => s.poll_shutdown(cx),
        }
    }
}

impl AsyncRead for AnyStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        match self.project() {
            StreamProj::Http(s) => s.poll_read(cx, buf),
            StreamProj::Https(s) => s.poll_read(cx, buf),
            StreamProj::HttpProxy(s) => s.poll_read(cx, buf),
            StreamProj::HttpsProxy(s) => s.poll_read(cx, buf),
            StreamProj::HttpTunnel(s) => s.poll_read(cx, buf),
        }
    }
}

impl<B> SendRequest<B>
where
    B: Body + 'static,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub async fn send_request(
        self,
        mut req: Request<B>,
    ) -> std::result::Result<Response<IncomingBody>, Error> {
        let SendRequest {
            mut sender,
            conn,
            host,
            proxy,
            auth,
        } = self;

        if proxy.is_some() {
            if let Some(auth) = auth {
                let auth_headers = auth.step(None)?;
                req.headers_mut().extend(auth_headers);
            }
        } else {
            // if not proxied, remove the authority part of the URI
            if req.method() != http::Method::CONNECT {
                let uri = req
                    .uri()
                    .path_and_query()
                    .cloned()
                    .unwrap_or_else(|| PathAndQuery::from_static("/"));
                *req.uri_mut() = Uri::from(uri);
            }
        }

        if !req.headers().contains_key(HOST) {
            if let Some(host) = &host {
                req.headers_mut().insert(HOST, HeaderValue::from_str(host)?);
            }
        }
        // We do not support connection pooling as of now.
        req.headers_mut()
            .insert(CONNECTION, HeaderValue::from_static("close"));

        let send_request = async move { sender.send_request(req).await };

        let (response, _connection) = tokio::join!(send_request, conn.with_upgrades());

        Ok(response?)
    }
}

async fn http_connect<T: AsyncRead + AsyncWrite + Send + Unpin + 'static>(
    stream: T,
    proxy: &HostAndPort,
    auth: &AuthenticatorFactory,
    dst: &HostAndPort,
) -> std::io::Result<Upgraded> {
    let (mut request_sender, connection) =
        http1::handshake(TokioIo::new(stream)).await.map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("HTTP handshake error: {e}"),
            )
        })?;

    let dst_uri = Uri::builder()
        .authority(dst.to_string())
        .build()
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Invalid authority '{dst}': {e}"),
            )
        })?;
    let mut request = Request::connect(dst_uri).header(HOST, dst.host());
    let auth = auth.make(proxy.host()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unable to build authenticator for '{proxy}': {e}"),
        )
    })?;
    let auth_headers = auth.step(None).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Unable to step authenticator for '{proxy}': {e}"),
        )
    })?;
    if let Some(headers) = request.headers_mut() {
        headers.extend(auth_headers)
    }
    let request = request.body(Empty::<Bytes>::new()).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Invalid HTTP request: {e}"),
        )
    })?;

    let send_request = async move {
        let response = request_sender.send_request(request).await.map_err(|e| {
            std::io::Error::other(format!("send request error from {proxy} for {dst}: {e}"))
        })?;
        let status = response.status();
        hyper::upgrade::on(response).await.map_err(|e| {
            std::io::Error::other(format!("HTTP {status} from {proxy} for {dst} error: {e}"))
        })
    };

    let (response, _connection) = tokio::join!(send_request, connection.with_upgrades());

    response
}
