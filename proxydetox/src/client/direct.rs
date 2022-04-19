use std::future::Future;
use std::pin::Pin;

use http::{Request, Uri};
use hyper::{client, Body};

type ClientService = client::service::Connect<client::connect::HttpConnector, Body, Uri>;
type HyperSendRequest = client::conn::SendRequest<Body>;

pub struct Direct(ClientService);

impl Direct {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for Direct {
    fn default() -> Self {
        let client = client::service::Connect::new(
            client::connect::HttpConnector::new(),
            client::conn::Builder::new(),
        );

        Self(client)
    }
}

impl tower::Service<Uri> for Direct {
    type Response = SendRequest;
    type Error = <ClientService as tower::Service<Uri>>::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let send_request = self.0.call(dst);
        let res = async move {
            let send_request = send_request.await?;
            Ok(SendRequest(send_request))
        };
        Box::pin(res)
    }
}

pub struct SendRequest(HyperSendRequest);

impl tower::Service<Request<Body>> for SendRequest {
    type Response = <HyperSendRequest as tower::Service<Request<Body>>>::Response;
    type Error = <HyperSendRequest as tower::Service<Request<Body>>>::Error;
    type Future = <HyperSendRequest as tower::Service<Request<Body>>>::Future;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::result::Result<(), Self::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<Body>) -> Self::Future {
        if req.method() != http::Method::CONNECT {
            // strip the authority, since direct clients to not expect this
            *req.uri_mut() = Uri::builder()
                .path_and_query(
                    req.uri()
                        .path_and_query()
                        .cloned()
                        .unwrap_or_else(|| http::uri::PathAndQuery::from_static("/")),
                )
                .build()
                .expect("request with valid URI expected");
        }
        self.0.send_request(req)
    }
}
