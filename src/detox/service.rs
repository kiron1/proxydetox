use std::result::Result;
use std::task::{self, Poll};

use futures_util::future;
use hyper::service::Service;
use tracing::{event, Level};
use tracing_attributes::instrument;

use crate::detox::DetoxSession;

// https://github.com/hyperium/hyper/blob/master/examples/tower_server.rs
#[derive(Clone)]
pub struct DetoxService {
    /// The master session where we take clones from
    session: DetoxSession,
}

impl std::fmt::Debug for DetoxService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DetoxService").finish()
    }
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

    #[instrument]
    fn call(&mut self, socket: &hyper::server::conn::AddrStream) -> Self::Future {
        event!(
            Level::DEBUG,
            remote_addr = %socket.remote_addr(),
            "New client {}", socket.remote_addr()
        );
        future::ok(self.session.clone())
    }
}
