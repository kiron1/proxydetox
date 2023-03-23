use hyper::body::Buf;
use hyper::{body::Bytes, Body};
use hyper_rustls::HttpsConnector;
use paclib::Proxy;
use proxy_client::HttpConnectConnector;
use std::io::Read;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tokio_rustls::TlsConnector;

pub struct Client {
    remote_addr: http::Uri,
    client: hyper::Client<HttpsConnector<HttpConnectConnector>, Body>,
    timeout: Duration,
}

impl Client {
    pub fn new(remote_uri: http::Uri, proxy: Proxy) -> Self {
        let tls_config = default_tls_config();
        let http_proxy = HttpConnectConnector::new(proxy, TlsConnector::from(tls_config));
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_native_roots()
            .https_only()
            .enable_http1()
            .wrap_connector(http_proxy);
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let timeout = Duration::from_millis(1500);

        Self {
            remote_addr: remote_uri,
            client,
            timeout,
        }
    }
    pub async fn request(&self, buf: Vec<u8>) -> std::result::Result<Vec<u8>, crate::error::Error> {
        let buf = Bytes::from(buf);

        let req = http::Request::post(self.remote_addr.clone())
            .header(http::header::ACCEPT, "application/dns-message")
            .header(http::header::CONTENT_TYPE, "application/dns-message")
            .body(Body::from(buf))?;
        let resp = timeout(self.timeout, self.client.request(req)).await??;

        let body = read_to_end(resp).await?;

        Ok(body)
    }
}

pub async fn read_to_end(res: http::Response<Body>) -> std::io::Result<Vec<u8>> {
    let body = hyper::body::aggregate(res)
        .await
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("aggregate: {e}")))?;
    let mut data = Vec::new();
    body.reader().read_to_end(&mut data)?;
    Ok(data)
}

fn default_tls_config() -> Arc<rustls::ClientConfig> {
    let mut roots = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().expect("load platform certs") {
        roots.add(&rustls::Certificate(cert.0)).unwrap();
    }

    let config = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(roots)
        .with_no_client_auth();

    Arc::new(config)
}
