use crate::Metered;
use std::io::ErrorKind;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncWrite};

#[derive(thiserror::Error, Debug)]
#[error(
    "Error during bidirectional: {duration:?} sec, {upstream_in} upsteam in, {upstream_out} upstream out, {downstream_in} downstream in, {downstream_out} downstream out: {source}"
)]
struct BytesLost {
    duration: Duration,
    upstream_in: u64,
    upstream_out: u64,
    downstream_in: u64,
    downstream_out: u64,
    #[source]
    source: std::io::Error,
}

/// Calls tokio::io::copy_bidirectional but ignores some of the common errors.
pub async fn copy_bidirectional<A, B>(upstream: &mut A, downstream: &mut B) -> std::io::Result<()>
where
    A: AsyncRead + AsyncWrite + Unpin + ?Sized,
    B: AsyncRead + AsyncWrite + Unpin + ?Sized,
{
    let mut upstream = Metered::new(upstream);
    let mut downstream = Metered::new(downstream);
    let begin = Instant::now();
    let cp = tokio::io::copy_bidirectional(&mut upstream, &mut downstream)
        .await
        .map(|_| ());

    let dt = Instant::now() - begin;
    let upstream_in = upstream.bytes_read();
    let upstream_out = upstream.bytes_written();
    let downstream_in = downstream.bytes_read();
    let downstream_out = downstream.bytes_written();
    let bytes_lost = upstream_in != downstream_out || upstream_out != downstream_in;

    // Ignore errors which we cannot influence (e.g. peer is terminating the
    // connection without a clean shutdown/close)
    match cp {
        Ok(_) => Ok(()),
        Err(e) => match e.kind() {
            ErrorKind::ConnectionReset | ErrorKind::BrokenPipe => Ok(()),
            ErrorKind::NotConnected => {
                // https://github.com/tokio-rs/tokio/issues/4674
                if bytes_lost {
                    Err(std::io::Error::other(BytesLost {
                        duration: dt,
                        upstream_in,
                        upstream_out,
                        downstream_in,
                        downstream_out,
                        source: e,
                    }))
                } else {
                    Ok(())
                }
            }
            _ => Err(e),
        },
    }
}
