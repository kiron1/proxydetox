use tokio::net::TcpStream;

pub async fn dial(uri: &http::Uri) -> tokio::io::Result<TcpStream> {
    match (uri.host(), uri.port_u16()) {
        (Some(host), Some(port)) => TcpStream::connect((host, port)).await,
        (_, _) => Err(tokio::io::Error::new(
            tokio::io::ErrorKind::AddrNotAvailable,
            "invalid URI",
        )),
    }
}
