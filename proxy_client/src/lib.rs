pub mod http_connect_connector;
pub mod http_connect_stream;
pub mod http_proxy_connector;
pub mod http_proxy_stream;

pub use http_proxy_connector::HttpProxyConnector;
pub use http_proxy_stream::HttpProxyInfo;

pub use http_connect_connector::HttpConnectConnector;
pub use http_connect_stream::HttpConnectStream;

pub type Client = hyper::Client<HttpProxyConnector, hyper::Body>;
