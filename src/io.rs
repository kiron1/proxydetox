use futures_util::future::try_join;
use tokio::io::{AsyncRead, AsyncWrite};

// Bidirectionl copy two async streams
pub async fn tunnel<T1, T2>(server: T1, client: T2) -> tokio::io::Result<(u64, u64)>
where
    T1: AsyncRead + AsyncWrite,
    T2: AsyncRead + AsyncWrite,
{
    // Proxying data
    let amounts = {
        let (mut server_rd, mut server_wr) = tokio::io::split(server);
        let (mut client_rd, mut client_wr) = tokio::io::split(client);

        let client_to_server = tokio::io::copy(&mut client_rd, &mut server_wr);
        let server_to_client = tokio::io::copy(&mut server_rd, &mut client_wr);

        try_join(client_to_server, server_to_client).await
    };

    match amounts {
        Ok((from_client, from_server)) => {
            log::trace!(
                "client wrote {} bytes and received {} bytes",
                from_client,
                from_server
            );
        }
        Err(ref cause) => {
            log::error!("tunnel error: {:?}", cause);
        }
    };
    amounts
}
