mod environment;

use crate::environment::{tcp, Environment};
use http::Request;
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_connect_direct() {
    let env = Environment::new().await;
    let server1 = tcp::Server::new(|mut s| {
        async move {
            let mut buf = [0u8; 4];
            s.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"PING");

            let buf = b"PONG";
            s.write_all(buf).await.unwrap();
            s.shutdown().await.unwrap();

            loop {
                // wait for close on client side
                let mut buf = [0u8; 128];
                let n = s.read(&mut buf).await.unwrap();
                if n == 0 {
                    break;
                }
            }
        }
    })
    .await;

    let req = Request::connect(server1.origin())
        .body(crate::environment::empty())
        .unwrap();

    let (status, _headers, upgraded) = env.connect(req).await;
    assert_eq!(status, http::StatusCode::OK);
    let mut upgraded = TokioIo::new(upgraded.expect("upgraded"));

    upgraded.write_all(b"PING").await.unwrap();
    upgraded.flush().await.unwrap();

    let mut buf = [0u8; 4];
    upgraded.read_exact(&mut buf).await.unwrap();
    assert_eq!(&buf, b"PONG");

    upgraded.shutdown().await.unwrap();
    drop(upgraded);

    tokio::join!(env.shutdown(), server1.shutdown());
}
