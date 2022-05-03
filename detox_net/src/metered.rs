use std::{io::Result, pin::Pin, task, task::Poll};
use tokio::io::{AsyncRead, AsyncWrite};

pub struct Metered<T> {
    stream: T,
    bytes_in: u64,
    bytes_out: u64,
}

impl<T> Metered<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            bytes_in: Default::default(),
            bytes_out: Default::default(),
        }
    }

    pub fn bytes_read(&self) -> u64 {
        self.bytes_in
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_out
    }
}

impl<T> AsyncWrite for Metered<T>
where
    T: AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize>> {
        let this = self.get_mut();
        let result = Pin::new(&mut this.stream).poll_write(cx, buf);
        if let Poll::Ready(Ok(size)) = result {
            *Pin::new(&mut this.bytes_out) += size as u64;
        }
        result
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}

impl<T> AsyncRead for Metered<T>
where
    T: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        let this = self.get_mut();
        let filled = buf.filled().len();
        let result = Pin::new(&mut this.stream).poll_read(cx, buf);
        let filled = buf.filled().len() - filled;
        *Pin::new(&mut this.bytes_in) += filled as u64;
        result
    }
}
