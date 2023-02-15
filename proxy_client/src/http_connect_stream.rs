use std::{io::Result, net::SocketAddr, pin::Pin, task, task::Poll};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::HttpProxyInfo;

pub struct HttpConnectStream {
    reader: Box<dyn AsyncRead + Send + Unpin>,
    writer: Box<dyn AsyncWrite + Send + Unpin>,
    local_addr: Option<SocketAddr>,
    remote_addr: Option<SocketAddr>,
}

impl HttpConnectStream {
    pub fn new(
        reader: Box<dyn AsyncRead + Send + Unpin>,
        writer: Box<dyn AsyncWrite + Send + Unpin>,
    ) -> Self {
        Self {
            reader,
            writer,
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
            reader: self.reader,
            writer: self.writer,
            local_addr,
            remote_addr,
        }
    }
}

impl hyper::client::connect::Connection for HttpConnectStream {
    fn connected(&self) -> hyper::client::connect::Connected {
        let connected = hyper::client::connect::Connected::new();
        match (self.local_addr, self.remote_addr) {
            (Some(local_addr), Some(remote_addr)) => connected.extra(HttpProxyInfo {
                remote_addr,
                local_addr,
            }),
            _ => connected,
        }
    }
}

impl AsyncWrite for HttpConnectStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        Pin::new(&mut self.get_mut().writer).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().writer).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().writer).poll_shutdown(cx)
    }
}

impl AsyncRead for HttpConnectStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().reader).poll_read(cx, buf)
    }
}
