mod environment;

use crate::environment::Environment;
use http::{header::PROXY_AUTHORIZATION, Request, Response};
use hyper::Body;
use proxydetox::net::read_to_string;
use tokio::join;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_get_request() {
    let http1 = environment::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        assert!(r.uri().authority().is_none());
        assert_eq!(r.uri().path(), "/text1.html");
        assert!(r.headers().get(PROXY_AUTHORIZATION).is_none());
        Response::builder()
            .body(Body::from(String::from("Hello World!")))
            .unwrap()
    });
    let env = Environment::new();

    let req = Request::get(http1.uri().path_and_query("/text1.html").build().unwrap())
        .body(Body::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = read_to_string(resp).await.unwrap();
    assert_eq!(body, "Hello World!");

    join!(env.shutdown(), http1.shutdown());
}
