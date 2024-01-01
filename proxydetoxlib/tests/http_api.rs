mod environment;

use std::io::Read;

use crate::environment::Environment;
use bytes::Buf;
use http::{
    header::{CONTENT_TYPE, HOST},
    Request,
};
use http_body_util::BodyExt;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn api_get_request() {
    let env = Environment::new().await;

    let req = Request::get("/").body(crate::environment::empty()).unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);

    env.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn api_invalid_request() {
    let env = Environment::new().await;

    let req = Request::options(env.proxy_uri().path_and_query("/").build().unwrap())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_ne!(resp.status(), http::StatusCode::OK);

    env.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn api_get_proxy_pac() {
    let env = Environment::new().await;

    let req = Request::get("/proxy.pac")
        .header(HOST, env.proxy_addr().to_string())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/x-ns-proxy-autoconfig")
    );
    let body = resp
        .into_body()
        .collect()
        .await
        .expect("receive body")
        .aggregate();
    let mut data = Vec::new();
    body.reader().read_to_end(&mut data).expect("read_to_end");
    let body = String::from_utf8(data).expect("UTF-8 data");
    assert_eq!(
        body,
        format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}\n",
            env.proxy_uri()
                .path_and_query("/")
                .build()
                .unwrap()
                .authority()
                .unwrap()
        )
    );

    env.shutdown().await;
}
