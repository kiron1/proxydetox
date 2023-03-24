#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod options;

use futures_util::stream::{FuturesUnordered, StreamExt};
use options::{Authorization, Options};
use proxydetoxlib::auth::netrc;
use proxydetoxlib::auth::AuthenticatorFactory;
use proxydetoxlib::socket;
use std::fs::File;
use std::result::Result;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::filter::EnvFilter;

#[cfg(static_library)]
#[no_mangle]
pub extern "C" fn main() {
    let config = Options::load_without_rcfile();
    if let Err(error) = run(&config) {
        tracing::error!(%error, "fatal error");
        write_error(&mut std::io::stderr(), error).ok();
        std::process::exit(1);
    }
}

#[cfg(not(static_library))]
fn main() {
    let config = Options::load();
    if let Err(error) = run(&config) {
        tracing::error!(%error, "fatal error");
        write_error(&mut std::io::stderr(), error).ok();
        std::process::exit(1);
    }
}

fn write_error<W, E: 'static>(writer: &mut W, error: E) -> std::io::Result<()>
where
    E: std::error::Error + Send + Sync,
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
async fn run(config: &Options) -> Result<(), proxydetoxlib::Error> {
    let env_name = format!("{}_LOG", env!("CARGO_PKG_NAME").to_uppercase());

    let filter = if let Ok(filter) = EnvFilter::try_from_env(&env_name) {
        filter
    } else {
        EnvFilter::default()
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

    let pac_script = if let Some(ref pac_file) = config.pac_file {
        let pac_script = pac_file.contents().await;
        if let Err(ref cause) = pac_script {
            tracing::error!(%cause, "PAC file error, will use default PAC script");
        }
        pac_script.ok()
    } else {
        None
    };

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

    let session = proxydetoxlib::Session::builder()
        .pac_script(pac_script)
        .authenticator_factory(Some(auth.clone()))
        .always_use_connect(config.always_use_connect)
        .connect_timeout(config.connect_timeout)
        .direct_fallback(config.direct_fallback)
        .client_tcp_keepalive(config.client_tcp_keepalive.clone())
        .build();

    let server = if let Some(name) = &config.activate_socket {
        socket::activate_socket(name)?
            .take()
            .into_iter()
            .map(hyper::Server::from_tcp)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        config
            .listen
            .iter()
            .map(hyper::Server::try_bind)
            .collect::<Result<Vec<_>, _>>()?
    };
    let server: Vec<_> = server
        .into_iter()
        .map(|s| s.tcp_keepalive(config.server_tcp_keepalive.time()))
        .map(|s| s.tcp_keepalive_interval(config.server_tcp_keepalive.time()))
        .map(|s| s.tcp_keepalive_retries(config.server_tcp_keepalive.retries()))
        .map({
            let s = session.clone();
            move |k| {
                let s = s.clone();
                k.serve(s)
            }
        })
        .collect();

    let addrs: Vec<_> = server.iter().map(|k| k.local_addr()).collect();
    let shutdown_token = CancellationToken::new();
    let server: FuturesUnordered<_> = server
        .into_iter()
        .map({
            let shutdown = shutdown_token.clone();
            move |k| {
                let shutdown = shutdown.clone();
                k.with_graceful_shutdown(async move { shutdown.cancelled().await })
            }
        })
        .collect();

    let timeout_token = CancellationToken::new();

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let graceful_shutdown_timeout = config.graceful_shutdown_timeout;
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;
        tokio::spawn({
            let timeout_token = timeout_token.clone();
            let shutdown_token = shutdown_token.clone();
            async move {
                tokio::select! {
                    _ = sigint.recv() => {},
                    _ = sigterm.recv() => {},
                }
                tracing::info!("triggering graceful shutdown");
                shutdown_token.cancel();
                tokio::time::sleep(graceful_shutdown_timeout).await;
                tracing::info!("graceful shutdown timeout");
                timeout_token.cancel();
            }
        });
    }
    #[cfg(not(unix))]
    {
        let graceful_shutdown_timeout = config.graceful_shutdown_timeout;
        tokio::spawn({
            let timeout_token = timeout_token.clone();
            let shutdown_token = shutdown_token.clone();
            async move {
                tokio::signal::ctrl_c().await.expect("ctrl_c event");
                tracing::info!("triggering graceful shutdown");
                shutdown_token.cancel();
                tokio::time::sleep(graceful_shutdown_timeout).await;
                tracing::info!("graceful shutdown timeout");
                timeout_token.cancel();
            }
        });
    }

    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut sighup = signal(SignalKind::hangup())?;
        let mut sigusr1 = signal(SignalKind::user_defined1())?;
        let pac_file = config.pac_file.clone();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = sighup.recv() => {
                        let pac_script = if let Some(ref pac_file) = pac_file {
                            let pac_script = pac_file.contents().await;
                            if let Err(ref cause) = pac_script {
                                tracing::error!(%cause, "PAC file error, will use default PAC script");
                            }
                            pac_script.ok()
                        } else {
                            None
                        };
                        session.set_pac_script(pac_script).await.ok();
                    },
                    _ = sigusr1.recv() => { session.set_pac_script(None).await.ok();},
                }
            }
        });
    }

    tracing::info!(listening=?addrs, authenticator=%auth, pac_file=?config.pac_file, "starting");
    tokio::pin!(server);
    tokio::pin!(timeout_token);
    loop {
        tokio::select! {
            s = server.next() => {
                if let Some(s) = s{
                    if let Err(cause) = s {
                        tracing::error!(%cause, "fatal server error");
                    }
                } else {
                    break;
                }
            },
            _ = timeout_token.cancelled() => break,
        }
    }
    Ok(())
}
