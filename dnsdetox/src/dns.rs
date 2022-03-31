use std::{pin::Pin, sync::Arc, task::Poll, time::Duration};

use crate::error::Result;
use futures_util::Stream;
use std::net::SocketAddr;
use tokio::{io::ReadBuf, net::UdpSocket, time::timeout};

const MAX_DATAGRAM_SIZE: usize = 65_507;

pub struct Client {
    // socket: UdpSocket,
    remote_addr: SocketAddr,
    timeout: Duration,
}

impl Client {
    pub fn new(remote_addr: SocketAddr) -> Self {
        let timeout = Duration::from_millis(500);
        Self {
            remote_addr,
            timeout,
        }
    }

    pub async fn request(&self, buf: &[u8]) -> Result<Vec<u8>> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(self.remote_addr).await?;

        let send_bytes = timeout(self.timeout, socket.send(buf)).await??;
        if send_bytes != buf.len() {
            log::error!("send bytes {} != {}", send_bytes, buf.len());
        }
        let mut buf = vec![0; MAX_DATAGRAM_SIZE];
        let n = timeout(self.timeout, socket.recv(&mut buf)).await??;
        buf.truncate(n);

        Ok(buf)
    }
}

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn serve(&self) -> Result<RequestStream> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let socket = UdpSocket::bind(addr).await?;
        let socket = Arc::new(socket);
        let data = vec![0; MAX_DATAGRAM_SIZE];

        Ok(RequestStream { socket, data })
    }
}

pub struct RequestStream {
    socket: Arc<UdpSocket>,
    data: Vec<u8>,
}

impl Stream for RequestStream {
    type Item = Result<(ClientRef, Vec<u8>)>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let Self {
            socket,
            ref mut data,
        } = self.get_mut();
        let mut data = Pin::new(data);
        let mut buffer = ReadBuf::new(&mut data);
        match Pin::new(&socket).poll_recv_from(cx, &mut buffer) {
            Poll::Ready(Ok(from)) => {
                let client_ref = ClientRef {
                    socket: socket.clone(),
                    remote_addr: from,
                };
                let data = Vec::from(buffer.filled());
                let result = Ok((client_ref, data));
                Poll::Ready(Some(result))
            }
            Poll::Ready(Err(cause)) => Poll::Ready(Some(Err(cause.into()))),

            Poll::Pending => Poll::Pending,
        }
    }
}

pub struct ClientRef {
    socket: Arc<UdpSocket>,
    remote_addr: SocketAddr,
}

impl ClientRef {
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    pub async fn reply(&self, data: &[u8]) -> Result<()> {
        self.socket.send_to(data, self.remote_addr).await?;
        Ok(())
    }
}
