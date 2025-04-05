use detox_auth::AuthenticatorFactory;
use detox_hyper::conn::Connection;
use detox_net::HostAndPort;
use http::StatusCode;
use http_body_util::{BodyExt, Full};
use hyper::body::{Buf, Bytes};
use paclib::Proxy;
use std::future::IntoFuture;
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct Client {
    remote_uri: http::Uri,
    proxy: Proxy,
    timeout: Duration,
}

impl Client {
    pub fn new(remote_uri: http::Uri, proxy: Proxy) -> Self {
        let timeout = Duration::from_millis(1500);

        Self {
            remote_uri,
            proxy,
            timeout,
        }
    }

    pub async fn request(&self, buf: Vec<u8>) -> std::result::Result<Vec<u8>, crate::error::Error> {
        let buf = Bytes::from(buf);

        let req: http::Request<http_body_util::Full<Bytes>> =
            http::Request::post(self.remote_uri.clone())
                .header(http::header::ACCEPT, "application/dns-message")
                .header(http::header::CONTENT_TYPE, "application/dns-message")
                .body(Full::from(buf))?;

        let dst = HostAndPort::try_from_uri(&self.remote_uri)?;
        let conn = Connection::http_tunnel(
            self.proxy.clone(),
            default_tls_config(),
            AuthenticatorFactory::none(),
            dst,
        );

        let conn = timeout(self.timeout, conn.into_future()).await??;
        let request_sender = conn.handshake().await?;
        let resp = timeout(self.timeout, request_sender.send_request(req)).await??;
        let (parts, body) = resp.into_parts();
        if parts.status != StatusCode::OK {
            return Err(crate::error::Error::UnexpectedHttpStatusCode(parts.status));
        }
        let body = body.collect().await?.aggregate();
        let mut data = Vec::new();
        body.reader().read_to_end(&mut data)?;
        Ok(data)
    }
}

fn default_tls_config() -> Arc<rustls::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("load platform certs") {
        roots.add(cert).unwrap();
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    Arc::new(config)
}
