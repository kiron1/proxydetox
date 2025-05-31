mod environment;

use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use crate::environment::httpd;
use detox_hyper::http::http_file;
use http::{Response, StatusCode, header::LOCATION};

static INIT: std::sync::Once = std::sync::Once::new();

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_file_too_many_redirects() {
    let server_origin = Arc::new(Mutex::new(Option::<String>::None));
    let counter = Arc::new(AtomicUsize::new(1));
    let http1 = httpd::Server::new({
        let uri = server_origin.clone();
        let counter = counter.clone();
        move |r| {
            let k = counter.fetch_add(1, Ordering::SeqCst);
            assert_eq!(r.method(), http::method::Method::GET);
            Response::builder()
                .status(StatusCode::PERMANENT_REDIRECT)
                .header(
                    LOCATION,
                    format!("http://{}/{}", uri.lock().unwrap().as_ref().unwrap(), k),
                )
                .body(crate::environment::empty())
                .unwrap()
        }
    })
    .await;

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

    let file = http_file(
        http1.uri().path_and_query("/text1.html").build().unwrap(),
        default_tls_config(),
    )
    .await;

    assert!(counter.load(Ordering::SeqCst) > 8);

    assert!(file.is_err());

    http1.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_file_redirect() {
    let server_origin = Arc::new(Mutex::new(Option::<String>::None));
    let http1 = httpd::Server::new({
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
                    .body(crate::environment::empty())
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::OK)
                    .body(crate::environment::full("Hello World!"))
                    .unwrap()
            }
        }
    })
    .await;

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

    let file = http_file(
        http1.uri().path_and_query("/").build().unwrap(),
        default_tls_config(),
    )
    .await
    .unwrap();

    assert_eq!(file, "Hello World!");

    http1.shutdown().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_file_ok() {
    let http1 = httpd::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::GET);
        Response::builder()
            .status(StatusCode::OK)
            .body(crate::environment::full("Hello World!"))
            .unwrap()
    })
    .await;

    let file = http_file(
        http1.uri().path_and_query("/text1.html").build().unwrap(),
        default_tls_config(),
    )
    .await
    .unwrap();

    assert_eq!(file, "Hello World!");

    http1.shutdown().await;
}

fn default_tls_config() -> Arc<rustls::ClientConfig> {
    INIT.call_once(|| {
        rustls::crypto::CryptoProvider::install_default(
            rustls::crypto::aws_lc_rs::default_provider(),
        )
        .expect("CryptoProvider::install_default");
    });
    let cfg = rustls::ClientConfig::builder()
        .with_root_certificates(rustls::RootCertStore::empty())
        .with_no_client_auth();
    Arc::new(cfg)
}
