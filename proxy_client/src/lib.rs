pub mod http_connect_connector;
pub mod http_connect_stream;
pub mod http_proxy_connector;
pub mod http_proxy_stream;
mod stream;

use std::net::SocketAddr;

pub use http_proxy_connector::HttpProxyConnector;
pub use http_proxy_stream::HttpProxyStream;

pub use http_connect_connector::HttpConnectConnector;
pub use http_connect_stream::HttpConnectStream;

pub type Client = hyper::Client<HttpProxyConnector, hyper::Body>;

#[derive(Clone, Debug)]
pub struct HttpProxyInfo {
    pub remote_addr: SocketAddr,
    pub local_addr: SocketAddr,
}

impl HttpProxyInfo {
    /// Get the remote address of the transport used.
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Get the local address of the transport used.
    pub fn local_addr(&self) -> SocketAddr {
        self.local_addr
    }
}
