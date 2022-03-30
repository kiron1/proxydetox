mod environment;

use crate::environment::Environment;
use http::Request;
use hyper::Body;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
async fn http_get_request() {
    let env = Environment::new();

    let req = Request::get(env.proxy_uri().path_and_query("/").build().unwrap())
        .body(Body::empty())
        .unwrap();

    let resp = env.send(req).await;

    assert_eq!(resp.status(), http::StatusCode::OK);

    env.shutdown().await;
}
