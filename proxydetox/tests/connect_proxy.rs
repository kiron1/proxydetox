mod environment;

use crate::environment::Environment;
use http::{header::PROXY_AUTHORIZATION, Request, Response, StatusCode};
use hyper::Body;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn connect_proxy_request_failure() {
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

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn connect_proxy_request() {
    let http1 = environment::Server::new(|mut r| {
        assert_eq!(r.method(), http::method::Method::CONNECT);
        assert!(r.uri().authority().is_some());
        assert!(r.uri().path_and_query().is_none());
        assert!(r.headers().get(PROXY_AUTHORIZATION).is_none());
        assert_eq!(r.method(), http::method::Method::CONNECT);

        tokio::task::spawn(async move {
            match hyper::upgrade::on(&mut r).await {
                Ok(mut client_upgraded) => {
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
                    std::thread::yield_now();

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

    let (resp, buf, mut stream) = env.connect(req).await;
    assert_eq!(resp.status(), http::StatusCode::OK);
    assert!(buf.is_empty());
    stream.write_all(b"PING").await.unwrap();
    let mut buf = [0u8; 4];
    stream.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"PONG");
    stream.shutdown().await.unwrap();

    join!(env.shutdown(), http1.shutdown());
}
