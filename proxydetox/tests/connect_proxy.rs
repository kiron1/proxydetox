mod environment;

use crate::environment::Environment;
use http::{header::PROXY_AUTHORIZATION, Request, Response, StatusCode};
use hyper::Body;
use tokio::{io::AsyncWriteExt, join};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn connect_proxy_request() {
    let http1 = environment::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::CONNECT);
        assert!(r.uri().authority().is_some());
        assert!(r.uri().path_and_query().is_none());
        assert!(r.headers().get(PROXY_AUTHORIZATION).is_none());
        assert_eq!(r.method(), http::method::Method::CONNECT);
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    });
    let env = Environment::builder()
        .pac_script(Some(format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}",
            http1.uri().build().unwrap().authority().unwrap()
        )))
        .build();

    let req = Request::connect(http1.uri().path_and_query("/").build().unwrap())
        .body(Body::empty())
        .unwrap();

    let (resp, _buf, mut stream) = env.connect(req).await;
    stream.shutdown().await.unwrap();

    assert_ne!(resp.status(), http::StatusCode::OK);

    join!(env.shutdown(), http1.shutdown());
}
