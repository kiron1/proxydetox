use std::result::Result;
use std::sync::Arc;
use std::task::{self, Poll};

use futures_util::future;
use hyper::service::Service;
use tokio::sync::Mutex;

use paclib::Evaluator;

use crate::detox::CreateDetoxError;
use crate::detox::DetoxSession;

// https://github.com/hyperium/hyper/blob/master/examples/tower_server.rs
#[derive(Clone)]
pub struct DetoxService {
    eval: Arc<Mutex<Evaluator>>,
    client: hyper::Client<hyper::client::HttpConnector>,
}

impl DetoxService {
    pub fn new(pac_script: &str) -> Self {
        let eval = Arc::new(Mutex::new(Evaluator::new(pac_script).unwrap()));
        let client = hyper::Client::new();
        DetoxService { client, eval }
    }

    fn make(&mut self) -> Result<DetoxSession, CreateDetoxError> {
        DetoxSession::new(self.eval.clone(), self.client.clone())
    }
}

impl<'a> Service<&'a hyper::server::conn::AddrStream> for DetoxService {
    type Response = DetoxSession;
    type Error = CreateDetoxError;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, socket: &hyper::server::conn::AddrStream) -> Self::Future {
        log::trace!("New client {}", socket.remote_addr());
        future::ok(self.make().unwrap())
    }
}
