use std::result::Result;
use std::task::{self, Poll};

use futures_util::future;
use hyper::service::Service as HyperService;
use tracing::{event, Level};
use tracing_attributes::instrument;

use crate::detox::Session;

// https://github.com/hyperium/hyper/blob/master/examples/tower_server.rs
#[derive(Clone)]
pub struct Service {
    /// The master session where we take clones from
    session: Session,
}

impl std::fmt::Debug for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Service").finish()
    }
}

impl Service {
    pub fn new(session: Session) -> Self {
        Service { session }
    }
}

impl<'a> HyperService<&'a hyper::server::conn::AddrStream> for Service {
    type Response = Session;
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
