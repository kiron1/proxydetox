mod environment;

use crate::environment::{httpd, read_to_string, Environment};
use http::{header::PROXY_AUTHORIZATION, Request, Response};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_get_request() {
    let http1 = httpd::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        assert!(r.uri().authority().is_none());
        assert_eq!(r.uri().path(), "/text1.html");
        assert!(r.headers().get(PROXY_AUTHORIZATION).is_none());
        Response::builder()
            .body(crate::environment::full(String::from("Hello World!")))
            .unwrap()
    })
    .await;
    let env = Environment::new().await;

    let req = Request::get(http1.uri().path_and_query("/text1.html").build().unwrap())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = read_to_string(resp.into_body()).await;
    assert_eq!(body, "Hello World!");

    tokio::join!(env.shutdown(), http1.shutdown());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn pac_script_error() {
    let http1 = httpd::Server::new(|_r| {
        {
            Response::builder()
                .body(crate::environment::empty())
                .unwrap()
        }
    })
    .await;
    let env = Environment::builder()
        .pac_script(Some(String::from("function brokenPacScript(url, host) {}")))
        .build()
        .await;

    let req = Request::get(http1.uri().build().unwrap())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);

    tokio::join!(env.shutdown(), http1.shutdown());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn pac_script_invalid_result() {
    let http1 = httpd::Server::new(|_r| {
        {
            Response::builder()
                .body(crate::environment::full(String::from("Hello World!")))
                .unwrap()
        }
    })
    .await;
    let env = Environment::builder()
        .pac_script(Some(String::from(
            "function FindProxyForURL(url, host) { return null; }",
        )))
        .build()
        .await;

    let req = Request::get(http1.uri().build().unwrap())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);

    tokio::join!(env.shutdown(), http1.shutdown());
}
