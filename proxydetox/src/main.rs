#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod options;

use detox_auth::netrc;
use detox_auth::AuthenticatorFactory;
use futures_util::future;
use futures_util::stream;
use options::{Authorization, Options};
use proxydetoxlib::server::Proxy;
use proxydetoxlib::socket;
use std::fs::File;
use std::result::Result;
use std::sync::Arc;
use tokio_stream::wrappers::TcpListenerStream;
use tracing_subscriber::filter::EnvFilter;

#[cfg(static_library)]
#[no_mangle]
pub extern "C" fn main() {
    let config = Options::load_without_rcfile();
    if let Err(error) = run(config) {
        tracing::error!(%error, "fatal error");
        write_error(&mut std::io::stderr(), error).ok();
        std::process::exit(1);
    }
}

#[cfg(not(static_library))]
fn main() {
    let config = Options::load();
    if let Err(error) = run(config) {
        tracing::error!(%error, "fatal error");
        write_error(&mut std::io::stderr(), error).ok();
        std::process::exit(1);
    }
}

fn write_error<W, E>(writer: &mut W, error: E) -> std::io::Result<()>
where
    E: std::error::Error + Send + Sync + 'static,
    W: std::io::Write,
{
    writeln!(writer, "fatal error: {error}")?;
    if let Some(cause) = error.source() {
        writeln!(writer, "Caused by:")?;
        for (i, e) in std::iter::successors(Some(cause), |e| e.source()).enumerate() {
            writeln!(writer, "{i}: {e}")?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn run(config: Arc<Options>) -> Result<(), proxydetoxlib::Error> {
    let env_name = format!("{}_LOG", env!("CARGO_PKG_NAME").to_uppercase());

    let filter = if let Ok(filter) = EnvFilter::try_from_env(&env_name) {
        filter
    } else {
        EnvFilter::default()
            .add_directive(
                format!("detox_auth={0}", config.log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("detox_hyper={0}", config.log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("detox_net={0}", config.log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("proxydetox={0}", config.log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("proxydetoxlib={0}", config.log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("paclib={0}", config.log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("proxy_client={0}", config.log_level)
                    .parse()
                    .expect("directive"),
            )
    };

    tracing_subscriber::fmt()
        .compact()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_env_filter(filter)
        .init();

    let auth = match &config.authorization {
        #[cfg(feature = "negotiate")]
        Authorization::Negotiate(ref negotiate) => {
            AuthenticatorFactory::negotiate(negotiate.clone())
        }
        #[cfg(not(feature = "negotiate"))]
        Authorization::Negotiate(_) => unreachable!(),
        Authorization::Basic(netrc_file) => {
            let store = if let Ok(file) = File::open(netrc_file) {
                netrc::Store::new(std::io::BufReader::new(file))?
            } else {
                netrc::Store::default()
            };
            AuthenticatorFactory::basic(store)
        }
    };
    tracing::debug!(%auth, "authorization");

    let context = proxydetoxlib::Context::builder()
        .pac_file(config.pac_file.clone())
        .authenticator_factory(Some(auth))
        .proxytunnel(config.proxytunnel)
        .connect_timeout(config.connect_timeout)
        .direct_fallback(config.direct_fallback)
        .client_tcp_keepalive(config.client_tcp_keepalive.clone())
        .build();

    let listeners = if let Some(name) = &config.activate_socket {
        socket::activate_socket(name)?
            .take()
            .into_iter()
            .inspect(|s: &std::net::TcpListener| {
                s.set_nonblocking(true).expect("nonblocking");
            })
            .map(tokio::net::TcpListener::from_std)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        future::join_all(config.listen.iter().map(tokio::net::TcpListener::bind))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?
    };

    let addrs = listeners
        .iter()
        .map(|k| k.local_addr())
        .collect::<Result<Vec<_>, _>>()?;

    let listeners = listeners
        .into_iter()
        .map(TcpListenerStream::new)
        .collect::<Vec<_>>();

    let listeners = stream::select_all(listeners);
    let (server, control) = Proxy::new(listeners, context.clone());

    tracing::info!(listening=?addrs, pac_file=?config.pac_file, "starting");

    // let server = std::pin::pin!(server);
    let server = tokio::spawn({
        let mut server = server;
        async move { server.run().await }
    });
    while !control.is_shutdown() {
        tokio::select! {
            _ = reload_trigger() => {
                context.load_pac_file(&config.pac_file).await?;
            },
            _ = direct_mode_trigger() => {
                context.load_pac_file(&None).await?;
            },
            _ = shutdown_trigger() => {
                tracing::info!("shutdown requested");
                control.shutdown();
                break;
            }
        }
    }

    let wait = control
        .wait_with_timeout(config.graceful_shutdown_timeout)
        .await;

    if let Err(cause) = server.await {
        tracing::warn!(%cause, "clean shutdown failed")
    }

    match wait {
        Ok(_) => tracing::info!("shutdown completed"),
        Err(cause) => tracing::warn!(%cause, "clean shutdown failed"),
    }

    Ok(())
}

#[cfg(unix)]
async fn reload_trigger() {
    use tokio::signal::unix::{signal, SignalKind};
    let sighup = signal(SignalKind::hangup());
    if let Ok(mut sighup) = sighup {
        sighup.recv().await;
    } else {
        future::pending::<Option<()>>().await;
    }
}

#[cfg(not(unix))]
async fn reload_trigger() {
    future::pending().await
}

#[cfg(unix)]
async fn direct_mode_trigger() {
    use tokio::signal::unix::{signal, SignalKind};
    let sigusr1 = signal(SignalKind::user_defined1());
    if let Ok(mut sigusr1) = sigusr1 {
        sigusr1.recv().await;
    } else {
        future::pending::<Option<()>>().await;
    }
}

#[cfg(not(unix))]
async fn direct_mode_trigger() {
    future::pending().await
}

#[cfg(unix)]
async fn shutdown_trigger() {
    use tokio::signal::unix::{signal, SignalKind};
    let signals = vec![
        signal(SignalKind::interrupt()),
        signal(SignalKind::terminate()),
    ];
    let signals = signals
        .into_iter()
        .filter_map(Result::ok)
        .map(|mut s| Box::pin(async move { s.recv().await }))
        .collect::<Vec<_>>();
    let _ = future::select_all(signals).await;
}

#[cfg(not(unix))]
async fn shutdown_trigger() {
    tokio::signal::ctrl_c().await.expect("ctrl_c event");
}
