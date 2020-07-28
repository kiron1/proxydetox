use std::result::Result;
use std::task::{self, Poll};

use futures_util::future;
use hyper::service::Service;

use crate::detox::DetoxSession;

// https://github.com/hyperium/hyper/blob/master/examples/tower_server.rs
#[derive(Clone)]
pub struct DetoxService {
    /// The master session where we take clones from
    session: DetoxSession,
}

impl DetoxService {
    pub fn new(pac_script: &str) -> Self {
        let session = DetoxSession::new(pac_script);
        DetoxService { session }
    }
}

impl<'a> Service<&'a hyper::server::conn::AddrStream> for DetoxService {
    type Response = DetoxSession;
    type Error = std::convert::Infallible;
    type Future = future::Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Ok(()).into()
    }

    fn call(&mut self, socket: &hyper::server::conn::AddrStream) -> Self::Future {
        log::trace!("New client {}", socket.remote_addr());
        future::ok(self.session.clone())
    }
}
