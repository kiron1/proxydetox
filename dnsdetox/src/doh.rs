use std::time::Duration;

use crate::error::Result;
use futures::stream::TryStreamExt;
use hyper::{body::Bytes, Body};
use hyper_tls::HttpsConnector;
use proxy_client::HttpProxyConnector;
use tokio::time::timeout;

pub struct Client {
    remote_addr: http::Uri,
    client: hyper::Client<HttpsConnector<HttpProxyConnector>, Body>,
    timeout: Duration,
}

impl Client {
    pub fn new(remote_uri: http::Uri, proxy_uri: http::Uri) -> Self {
        let http_proxy = proxy_client::HttpProxyConnector::new_with_connect(proxy_uri, true);
        let https = HttpsConnector::new_with_connector(http_proxy);
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let timeout = Duration::from_millis(1500);

        Self {
            remote_addr: remote_uri,
            client,
            timeout,
        }
    }
    pub async fn request(&self, buf: Vec<u8>) -> Result<Vec<u8>> {
        let buf = Bytes::from(buf);

        let req = http::Request::post(self.remote_addr.clone())
            .header(http::header::ACCEPT, "application/dns-message")
            .header(http::header::CONTENT_TYPE, "application/dns-message")
            .body(Body::from(buf))?;
        let mut resp = timeout(self.timeout, self.client.request(req)).await??;

        let body = resp.body_mut();
        let body: Vec<u8> = body
            .try_fold(Vec::new(), |mut vec, data| {
                vec.extend(data);
                futures::future::ok(vec)
            })
            .await
            .unwrap();
        Ok(body)
    }
}
