use hyper::body::Buf;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tracing_attributes::instrument;

#[instrument]
pub async fn dial(uri: &http::Uri) -> tokio::io::Result<TcpStream> {
    match (uri.host(), uri.port_u16()) {
        (Some(host), Some(port)) => TcpStream::connect((host, port)).await,
        (_, _) => Err(tokio::io::Error::new(
            tokio::io::ErrorKind::AddrNotAvailable,
            "invalid URI",
        )),
    }
}

pub async fn http_file(uri: http::Uri) -> std::io::Result<String> {
    let client = hyper::Client::new();
    let res = client
        .get(uri)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "GET"))?;
    let body = hyper::body::aggregate(res)
        .await
        .map_err(|_| Error::new(ErrorKind::Other, "aggregate"))?;
    let mut buffer = String::new();
    body.reader().read_to_string(&mut buffer)?;
    Ok(buffer)
}

#[cfg(target_os = "linux")]
pub fn original_destination_address(socket: &impl std::os::unix::io::AsRawFd) -> SocketAddr {
    // TODO: get acutuall target address
    // https://github.com/mitmproxy/mitmproxy/blob/main/mitmproxy/platform/linux.py
    // csock.getsockopt(socket.SOL_IP, SO_ORIGINAL_DST, 16)
    let fd = socket.as_raw_fd();
    // struct sockaddr_in addr;
    // bzero((char *) &addr, sizeof(addr));
    // addr.sin_family = AF_INET;
    // socklen_t addr_sr = sizeof(addr);
    // getsockopt(fd, SOL_IP, SO_ORIGINAL_DST, &addr, &addr_sz);
    let mut addr: libc::sockaddr_in = unsafe { std::mem::zeroed() };
    let mut optlen = std::mem::size_of_val(&addr) as libc::socklen_t;
    let rc = unsafe {
        libc::getsockopt(
            fd,
            libc::SOL_IP,
            libc::SO_ORIGINAL_DST,
            &mut addr as *mut _ as *mut _,
            &mut optlen as *mut libc::socklen_t,
        )
    };

    let port = u16::from_be(addr.sin_port);
    let addr = u32::from_be(addr.sin_addr.s_addr);

    let ip = [
        (addr >> 24) as u8 & 255u8,
        (addr >> 16) as u8 & 255u8,
        (addr >> 8) as u8 & 255u8,
        (addr >> 0) as u8 & 255u8,
    ];

    SocketAddr::from((ip, port))
}

#[cfg(not(target_os = "linux"))]
pub fn original_destination_address(_socket: &impl std::os::unix::io::AsRawFd) -> SocketAddr {
    todo!("Not implemented for this OS");
}
