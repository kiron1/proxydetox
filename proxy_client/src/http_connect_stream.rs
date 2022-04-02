use std::{io::Result, pin::Pin, task, task::Poll};
use tokio::{
    io::{AsyncRead, AsyncWrite, WriteHalf},
    net::TcpStream,
};

pub struct HttpConnectStream {
    reader: Box<dyn AsyncRead + Send + Unpin>,
    writer: WriteHalf<TcpStream>,
}

impl HttpConnectStream {
    pub fn new(reader: Box<dyn AsyncRead + Send + Unpin>, writer: WriteHalf<TcpStream>) -> Self {
        Self { reader, writer }
    }
}

impl hyper::client::connect::Connection for HttpConnectStream {
    fn connected(&self) -> hyper::client::connect::Connected {
        hyper::client::connect::Connected::new()
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
