use futures_util::future::try_join;
use tokio::io::AsyncWriteExt;
use tokio::io::{AsyncRead, AsyncWrite};

pub async fn copy_and_close<'a, R, W>(reader: &'a mut R, writer: &'a mut W) -> std::io::Result<u64>
where
    R: AsyncRead + Unpin + ?Sized,
    W: AsyncWrite + Unpin + ?Sized,
{
    let res = tokio::io::copy(reader, writer).await;
    if res.is_ok() {
        writer.flush().await?;
        writer.shutdown().await?;
    }
    res
}

// Bidirectionl copy two async streams
pub async fn tunnel<T1, T2>(server: T1, client: T2) -> tokio::io::Result<(u64, u64)>
where
    T1: AsyncRead + AsyncWrite,
    T2: AsyncRead + AsyncWrite,
{
    // Proxying data
    let (mut server_rd, mut server_wr) = tokio::io::split(server);
    let (mut client_rd, mut client_wr) = tokio::io::split(client);

    let client_to_server = copy_and_close(&mut client_rd, &mut server_wr);
    let server_to_client = copy_and_close(&mut server_rd, &mut client_wr);

    try_join(client_to_server, server_to_client).await
}
