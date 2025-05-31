mod environment;

use crate::environment::{Environment, httpd};
use http::{Request, Response, StatusCode, header::PROXY_AUTHORIZATION};
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn connect_proxy_request_failure() {
    let http1 = httpd::Server::new(|r| {
        assert_eq!(r.method(), http::method::Method::CONNECT);
        assert!(r.uri().authority().is_some());
        assert!(r.uri().path_and_query().is_none());
        assert!(r.headers().get(PROXY_AUTHORIZATION).is_none());
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(crate::environment::empty())
            .unwrap()
    })
    .await;

    let env = Environment::builder()
        .pac_script(Some(format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}",
            http1.host_and_port()
        )))
        .build()
        .await;

    let req = Request::connect(http1.host_and_port())
        .body(crate::environment::empty())
        .unwrap();

    let (status, _headers, upgraded) = env.connect(req).await;
    assert_eq!(status, http::StatusCode::BAD_GATEWAY);
    assert!(upgraded.is_err());

    tokio::join!(env.shutdown(), http1.shutdown());
}

fn proxy_request_handler(
    mut r: Request<hyper::body::Incoming>,
) -> Response<crate::environment::Body> {
    assert_eq!(r.method(), http::method::Method::CONNECT);
    assert!(r.uri().authority().is_some());
    assert!(r.uri().path_and_query().is_none());
    assert!(r.headers().get(PROXY_AUTHORIZATION).is_none());
    assert_eq!(r.method(), http::method::Method::CONNECT);

    tokio::task::spawn(async move {
        match hyper::upgrade::on(&mut r).await {
            Ok(client_upgraded) => {
                let (mut client, mut server) = tokio::io::duplex(64);

                tokio::spawn(async move {
                    let mut buf = [0u8; 4];
                    server.read_exact(&mut buf).await.unwrap();
                    assert_eq!(&buf, b"PING");

                    server.write_all(b"PONG").await.unwrap();

                    loop {
                        // wait for close on client side
                        let mut buf = [0u8; 128];
                        let n = server.read(&mut buf).await.unwrap();
                        if n == 0 {
                            break;
                        }
                    }
                });
                let mut client_upgraded = TokioIo::new(client_upgraded);

                if let Err(cause) =
                    tokio::io::copy_bidirectional(&mut client, &mut client_upgraded).await
                {
                    tracing::error!(?cause, "tunnel error")
                }
            }
            Err(cause) => tracing::error!(%cause, "upgrade error"),
        }
    });

    // Response with a OK to the client
    Response::builder()
        .status(StatusCode::OK)
        .body(crate::environment::empty())
        .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn connect_proxy_request() {
    let http1 = httpd::Server::new(proxy_request_handler).await;
    let env = Environment::builder()
        .pac_script(Some(format!(
            "function FindProxyForURL(url, host) {{ return \"PROXY {}\"; }}",
            http1.host_and_port()
        )))
        .build()
        .await;

    let req = Request::connect(http1.host_and_port())
        .body(crate::environment::empty())
        .unwrap();

    let (status, _headers, upgraded) = env.connect(req).await;
    assert_eq!(status, http::StatusCode::OK);
    let mut upgraded = TokioIo::new(upgraded.expect("upgraded"));
    upgraded.write_all(b"PING").await.unwrap();
    let mut buf = [0u8; 4];
    upgraded.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"PONG");
    upgraded.shutdown().await.unwrap();

    tokio::join!(env.shutdown(), http1.shutdown());
}
