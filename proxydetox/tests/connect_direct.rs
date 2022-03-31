mod environment;

use crate::environment::{tcp, Environment};
use http::Request;
use hyper::Body;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn http_connect_direct() {
    let env = Environment::new();
    let server1 = tcp::Server::new(|mut s| {
        tracing::trace!("handle created");
        async move {
            tracing::trace!("handle");
            let mut buf = [0u8, 0, 0, 0];
            s.read_exact(&mut buf).await.unwrap();
            tracing::trace!("got {:?}", buf);
            assert_eq!(&buf, b"PING");

            let buf = b"PONG";
            s.write_all(buf).await.unwrap();
            tokio::task::yield_now().await;

            loop {
                // wait for close on client side
                tracing::trace!("wait for close");
                let mut buf = [0u8; 128];
                let n = s.read(&mut buf).await.unwrap();
                tracing::trace!("wait for close: {}", n);
                if n == 0 {
                    break;
                }
            }
            s.shutdown().await.unwrap();
            tracing::trace!("handle done");
        }
    });

    let req = Request::connect(server1.origin())
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
    drop(stream);

    tokio::join!(env.shutdown(), server1.shutdown());
}
