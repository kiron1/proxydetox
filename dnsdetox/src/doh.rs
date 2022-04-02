use hyper::body::Buf;
use hyper::{body::Bytes, Body};
use hyper_tls::HttpsConnector;
use proxy_client::HttpConnectConnector;
use std::io::Read;
use std::time::Duration;
use tokio::time::timeout;

pub struct Client {
    remote_addr: http::Uri,
    client: hyper::Client<HttpsConnector<HttpConnectConnector>, Body>,
    timeout: Duration,
}

impl Client {
    pub fn new(remote_uri: http::Uri, proxy_uri: http::Uri) -> Self {
        let http_proxy = HttpConnectConnector::new(proxy_uri);
        let https = HttpsConnector::new_with_connector(http_proxy);
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
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("aggregate: {}", e)))?;
    let mut data = Vec::new();
    body.reader().read_to_end(&mut data)?;
    Ok(data)
}
