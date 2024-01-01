mod environment;

use crate::environment::{httpd, read_to_string, Environment};
use http::{header::PROXY_AUTHORIZATION, Request, Response, Uri};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_get_via_proxy_request() {
    let http1 = httpd::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        assert!(r.uri().authority().is_some());
        assert_eq!(r.uri().path(), "/text1.html");
        assert!(r.headers().get(PROXY_AUTHORIZATION).is_none());
        Response::builder()
            .body(crate::environment::full(String::from("Hello World!")))
            .unwrap()
    })
    .await;
    let env = Environment::builder()
        .pac_script(Some(format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}",
            http1.uri().build().unwrap().authority().unwrap()
        )))
        .build()
        .await;

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
async fn http_get_via_proxy_with_auth_request() {
    let proxy1 = httpd::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        assert!(r.uri().authority().is_some());
        assert_eq!(r.uri().path(), "/text1.html");
        assert_eq!(
            r.headers()
                .get(PROXY_AUTHORIZATION)
                .and_then(|v| v.to_str().ok()),
            Some("Basic aGVsbG86d29ybGQ=")
        );
        Response::builder()
            .body(crate::environment::full(String::from("Hello World!")))
            .unwrap()
    })
    .await;
    let env = Environment::builder()
        .pac_script(Some(format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}",
            proxy1.uri().build().unwrap().authority().unwrap()
        )))
        .netrc_content(Some(format!(
            "machine {}\nlogin {}\npassword {}\n",
            proxy1.uri().build().unwrap().host().unwrap(),
            "hello",
            "world"
        )))
        .build()
        .await;

    let req = Request::get("http://example.org/text1.html".parse::<Uri>().unwrap())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = read_to_string(resp.into_body()).await;
    assert_eq!(body, "Hello World!");

    tokio::join!(env.shutdown(), proxy1.shutdown());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_get_via_proxy_with_default_auth_request() {
    let proxy1 = httpd::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        assert!(r.uri().authority().is_some());
        assert_eq!(r.uri().path(), "/text1.html");
        assert_eq!(
            r.headers()
                .get(PROXY_AUTHORIZATION)
                .and_then(|v| v.to_str().ok()),
            Some("Basic aGVsbG86d29ybGQ=")
        );
        Response::builder()
            .body(crate::environment::full(String::from("Hello World!")))
            .unwrap()
    })
    .await;
    let env = Environment::builder()
        .pac_script(Some(format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}",
            proxy1.uri().build().unwrap().authority().unwrap()
        )))
        .netrc_content(Some(format!(
            "machine example.org\nlogin {}\npassword {}\ndefault login {} password {}\n",
            "invalid", "invalid", "hello", "world"
        )))
        .build()
        .await;

    let req = Request::get("http://example.org/text1.html".parse::<Uri>().unwrap())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);
    let body = read_to_string(resp.into_body()).await;
    assert_eq!(body, "Hello World!");

    tokio::join!(env.shutdown(), proxy1.shutdown());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_get_via_proxy_with_auth_request_407() {
    let proxy1 = httpd::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        assert!(r.uri().authority().is_some());
        assert_eq!(r.uri().path(), "/text1.html");
        assert!(r.headers().get(PROXY_AUTHORIZATION).is_some());
        Response::builder()
            .status(http::StatusCode::PROXY_AUTHENTICATION_REQUIRED)
            .body(crate::environment::empty())
            .unwrap()
    })
    .await;
    let env = Environment::builder()
        .pac_script(Some(format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}",
            proxy1.uri().build().unwrap().authority().unwrap()
        )))
        .netrc_content(Some(format!(
            "machine {}\nlogin {}\npassword {}\n",
            proxy1.uri().build().unwrap().host().unwrap(),
            "hello",
            "world"
        )))
        .build()
        .await;

    let req = Request::get("http://example.org/text1.html".parse::<Uri>().unwrap())
        .body(crate::environment::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::BAD_GATEWAY);

    tokio::join!(env.shutdown(), proxy1.shutdown());
}
