mod environment;

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use http::{header::LOCATION, Response, StatusCode};
use hyper::Body;
use proxydetoxlib::http_file;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_file_too_many_redirects() {
    let http1 = environment::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        Response::builder()
            .status(StatusCode::PERMANENT_REDIRECT)
            .header(LOCATION, "http://example.org/next.html")
            .body(Body::empty())
            .unwrap()
    });

    let file = http_file(http1.uri().path_and_query("/text1.html").build().unwrap()).await;

    assert!(file.is_err());

    http1.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_file_redirect() {
    let server_origin = Arc::new(Mutex::new(Option::<String>::None));
    let http1 = environment::Server::new({
        let uri = server_origin.clone();
        let redirects = ["/", "/1", "/2", "/3"];
        let counter = Arc::new(AtomicUsize::new(1));
        move |r| {
            let k = counter.fetch_add(1, Ordering::SeqCst);
            assert!(k < redirects.len());
            assert_eq!(r.method(), http::method::Method::GET);
            assert_eq!(r.uri(), redirects[k - 1]);
            if k < redirects.len() - 1 {
                Response::builder()
                    .status(StatusCode::PERMANENT_REDIRECT)
                    .header(
                        LOCATION,
                        format!(
                            "http://{}{}",
                            uri.lock().unwrap().as_ref().unwrap(),
                            redirects[k]
                        ),
                    )
                    .body(Body::empty())
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::OK)
                    .body(Body::from("Hello World!"))
                    .unwrap()
            }
        }
    });

    *server_origin.lock().unwrap() = Some(
        http1
            .uri()
            .path_and_query("/")
            .build()
            .unwrap()
            .authority()
            .unwrap()
            .as_str()
            .to_owned(),
    );

    let file = http_file(http1.uri().path_and_query("/").build().unwrap())
        .await
        .unwrap();

    assert_eq!(file, "Hello World!");

    http1.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_file_ok() {
    let http1 = environment::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("Hello World!"))
            .unwrap()
    });

    let file = http_file(http1.uri().path_and_query("/text1.html").build().unwrap())
        .await
        .unwrap();

    assert_eq!(file, "Hello World!");

    http1.shutdown().await;
}
