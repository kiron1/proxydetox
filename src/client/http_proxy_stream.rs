use hyper::client::connect::{Connected, Connection};
use std::{
    pin::Pin,
    task::{self, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};

pub struct HttpProxyStream {
    inner: TcpStream,
}

impl HttpProxyStream {
    pub fn new(stream: TcpStream) -> Self {
        Self { inner: stream }
    }
}

impl Connection for HttpProxyStream {
    fn connected(&self) -> Connected {
        let connected = Connected::new();
        let connected = connected.proxy(true);
        // if let Ok(remote_addr) = self.inner.peer_addr() {
        //     connected.extra(HttpInfo { remote_addr })
        // } else {
        //     connected
        // }
        connected
    }
}

// https://stackoverflow.com/a/56117052
// https://stackoverflow.com/a/57377607
// https://doc.rust-lang.org/std/pin/index.html#projections-and-structural-pinning
// https://docs.rs/pin-project/0.4.22/pin_project/index.html#examples

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
        buf: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().inner).poll_read(cx, buf)
    }
}
