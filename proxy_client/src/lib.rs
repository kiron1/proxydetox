pub mod http_proxy_connector;
pub mod http_proxy_stream;

pub use http_proxy_connector::HttpProxyConnector;
pub use http_proxy_stream::HttpProxyInfo;

pub type Client = hyper::Client<HttpProxyConnector, hyper::Body>;
