use futures_util::{future::try_join, TryFutureExt};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing_futures::Instrument;

pub fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

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

        let client_to_server = tokio::io::copy(&mut client_rd, &mut server_wr)
            .map_ok(|bytes_copied| {
                tracing::trace!(bytes_copied);
                bytes_copied
            })
            .map_err(|error| {
                tracing::error!(%error);
                error
            })
            .instrument(tracing::debug_span!("client_to_server"));
        let server_to_client = tokio::io::copy(&mut server_rd, &mut client_wr)
            .map_ok(|bytes_copied| {
                tracing::trace!(bytes_copied);
                bytes_copied
            })
            .map_err(|error| {
                tracing::error!(%error);
                error
            })
            .instrument(tracing::debug_span!("server_to_client"));

        try_join(client_to_server, server_to_client).await
    };

    amounts
}
