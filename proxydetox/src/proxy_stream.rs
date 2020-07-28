use bytes::{Buf, BufMut};
use std::io;
use std::mem::MaybeUninit;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite};

use hyper::client::connect::{Connected, Connection};

/// A Proxy Stream wrapper
pub enum ProxyStream<R> {
    Regular(R),
    #[cfg(any(feature = "tls", feature = "rustls"))]
    Secured(TlsStream<R>),
}

impl<R: AsyncRead + AsyncWrite + Unpin> AsyncRead for ProxyStream<R> {
    unsafe fn prepare_uninitialized_buffer(&self, buf: &mut [MaybeUninit<u8>]) -> bool {
        match *self {
            ProxyStream::Regular(ref s) => s.prepare_uninitialized_buffer(buf),

            #[cfg(any(feature = "tls", feature = "rustls"))]
            ProxyStream::Secured(ref s) => s.prepare_uninitialized_buffer(buf),
        }
    }

    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        match_fn_pinned!(self, poll_read, cx, buf)
    }

    fn poll_read_buf<B: BufMut>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>>
    where
        Self: Sized,
    {
        match_fn_pinned!(self, poll_read_buf, cx, buf)
    }
}

impl<R: AsyncRead + AsyncWrite + Unpin> AsyncWrite for ProxyStream<R> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match_fn_pinned!(self, poll_write, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match_fn_pinned!(self, poll_flush, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match_fn_pinned!(self, poll_shutdown, cx)
    }

    fn poll_write_buf<B: Buf>(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut B,
    ) -> Poll<io::Result<usize>> {
        match_fn_pinned!(self, poll_write_buf, cx, buf)
    }
}

impl<R: AsyncRead + AsyncWrite + Connection + Unpin> Connection for ProxyStream<R> {
    fn connected(&self) -> Connected {
        match self {
            ProxyStream::Regular(s) => s.connected(),
            #[cfg(feature = "tls")]
            ProxyStream::Secured(s) => s.get_ref().connected().proxy(true),

            #[cfg(feature = "rustls")]
            ProxyStream::Secured(s) => s.get_ref().0.connected().proxy(true),
        }
    }
}