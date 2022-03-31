mod environment;

use crate::environment::Environment;
use http::{header::CONTENT_TYPE, Request};
use hyper::Body;
use proxydetox::net::read_to_string;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn api_get_request() {
    let env = Environment::new();

    let req = Request::get(env.proxy_uri().path_and_query("/").build().unwrap())
        .body(Body::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);

    env.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn api_invalid_request() {
    let env = Environment::new();

    let req = Request::options(env.proxy_uri().path_and_query("/").build().unwrap())
        .body(Body::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_ne!(resp.status(), http::StatusCode::OK);

    env.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn api_get_proxy_pac() {
    let env = Environment::new();

    let req = Request::get(
        env.proxy_uri()
            .path_and_query("/proxy.pac")
            .build()
            .unwrap(),
    )
    .body(Body::empty())
    .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);
    assert_eq!(
        resp.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok()),
        Some("application/x-ns-proxy-autoconfig")
    );
    let body = read_to_string(resp).await.unwrap();
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
