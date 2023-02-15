use crate::{stream::MaybeTlsStream, HttpProxyInfo};
use hyper::client::connect::{Connected, Connection};
use std::{
    net::SocketAddr,
    pin::Pin,
    task::{self, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

pub struct HttpProxyStream {
    inner: MaybeTlsStream<TcpStream>,
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
}

impl HttpProxyStream {
    pub fn new(stream: MaybeTlsStream<TcpStream>) -> Self {
        Self {
            inner: stream,
            local_addr: None,
            remote_addr: None,
        }
    }

    pub fn with_addr(
        self,
        local_addr: Option<SocketAddr>,
        remote_addr: Option<SocketAddr>,
    ) -> Self {
        Self {
            inner: self.inner,
            local_addr,
            remote_addr,
        }
    }
}

impl Connection for HttpProxyStream {
    fn connected(&self) -> Connected {
        let connected = Connected::new().proxy(true);
        match (self.local_addr, self.remote_addr) {
            (Some(local_addr), Some(remote_addr)) => connected.extra(HttpProxyInfo {
                remote_addr,
                local_addr,
            }),
            _ => connected,
        }
    }
}

impl AsyncWrite for HttpProxyStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_write(cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        Pin::new(&mut self.get_mut().inner).poll_shutdown(cx)
    }
}

impl AsyncRead for HttpProxyStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}
