#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod options;

use detox_auth::AuthenticatorFactory;
use detox_auth::netrc;
use futures_util::future;
use futures_util::stream;
use options::{Authorization, Options};
use proxydetoxlib::{server::Server, socket};
use std::fs::File;
use std::net::IpAddr;
use std::result::Result;
use std::sync::Arc;
use tokio_stream::wrappers::TcpListenerStream;
use tracing_subscriber::filter::EnvFilter;

#[cfg(static_library)]
#[unsafe(no_mangle)]
pub extern "C" fn main() {
    let config = Options::load_without_rcfile();

    setup_tracing(&config.log_level, config.logfile());

    if let Err(error) = run(config) {
        tracing::error!(%error, "fatal error");
        write_error(&mut std::io::stderr(), error).ok();
        std::process::exit(1);
    }
}

#[cfg(not(static_library))]
fn main() {
    let config = Options::load();

    setup_tracing(&config.log_level, config.logfile());

    #[cfg(target_family = "windows")]
    if config.attach_console {
        unsafe {
            windows::Win32::System::Console::AttachConsole(
                windows::Win32::System::Console::ATTACH_PARENT_PROCESS,
            )
            .unwrap();
        }
    }

    rustls::crypto::CryptoProvider::install_default(rustls::crypto::aws_lc_rs::default_provider())
        .expect("CryptoProvider::install_default");

    if let Err(error) = run(config) {
        tracing::error!(%error, "fatal error");
        write_error(&mut std::io::stderr(), error).ok();
        std::process::exit(1);
    }
}

fn setup_tracing(log_level: &tracing::level_filters::LevelFilter, logfile: Option<File>) {
    let env_name = format!("{}_LOG", env!("CARGO_PKG_NAME").to_uppercase());

    let filter = match EnvFilter::try_from_env(&env_name) {
        Ok(filter) => filter,
        _ => EnvFilter::default()
            .add_directive(
                format!("detox_auth={0}", log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("detox_hyper={0}", log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("detox_net={0}", log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("proxydetox={0}", log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(
                format!("proxydetoxlib={0}", log_level)
                    .parse()
                    .expect("directive"),
            )
            .add_directive(format!("paclib={0}", log_level).parse().expect("directive"))
            .add_directive(
                format!("proxy_client={0}", log_level)
                    .parse()
                    .expect("directive"),
            ),
    };

    let fmt = tracing_subscriber::fmt()
        .compact()
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_env_filter(filter);
    match logfile {
        Some(f) => {
            fmt.with_writer(f).init();
        }
        _ => {
            fmt.with_writer(std::io::stderr).init();
        }
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
    let auth = match &config.authorization {
        #[cfg(feature = "negotiate")]
        Authorization::Negotiate(negotiate) => AuthenticatorFactory::negotiate(negotiate.clone()),
        #[cfg(not(feature = "negotiate"))]
        Authorization::Negotiate(_) => unreachable!(),
        Authorization::Basic(netrc_file) => {
            let store = match File::open(netrc_file) {
                Ok(file) => netrc::Store::new(std::io::BufReader::new(file))?,
                _ => netrc::Store::default(),
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
        .race_connect(config.race_connect)
        .parallel_connect(config.parallel_connect)
        .direct_fallback(config.direct_fallback)
        .client_tcp_keepalive(config.client_tcp_keepalive.clone())
        .build();

    if let Some(my_ip) = config.my_ip_address {
        context.set_my_ip_address(my_ip).await?;
    }

    let listeners = match &config.activate_socket {
        Some(name) => socket::activate_socket(name)?
            .take()
            .into_iter()
            .inspect(|s: &std::net::TcpListener| {
                s.set_nonblocking(true).expect("nonblocking");
            })
            .map(tokio::net::TcpListener::from_std)
            .collect::<Result<Vec<_>, _>>()?,
        _ => future::join_all(config.listen.iter().map(tokio::net::TcpListener::bind))
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()?,
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
    let (server, control) = Server::new(listeners, context.clone());

    tracing::info!(listening=?addrs, pac_file=?config.pac_file, "starting");

    let server = tokio::spawn(async move { server.run().await });
    tokio::pin!(server);
    let joiner = loop {
        tokio::select! {
            _ = reload_trigger() => {
                context.load_pac_file(&config.pac_file).await?;
                context.set_my_ip_address(my_ip_address()).await?;
            },
            _ = direct_mode_trigger() => {
                context.load_pac_file(&None).await?;
                context.set_my_ip_address(my_ip_address()).await?;
            },
            _ = shutdown_trigger() => {
                tracing::info!("shutdown requested");
                control.shutdown();
            }
            rc = &mut server => {
                if let Err(cause) = &rc {
                    tracing::error!(%cause, "server error");
                }
                break rc;
            }
        }
    };

    match joiner {
        Ok(Ok(joiner)) => {
            let wait = joiner
                .wait_with_timeout(config.graceful_shutdown_timeout)
                .await;
            if let Err(cause) = wait {
                tracing::error!(%cause, "graceful shutdown timeout");
            }
        }
        Ok(Err(cause)) => {
            tracing::error!(%cause, "server error");
        }
        Err(cause) => {
            tracing::error!(%cause, "spawn error");
        }
    }

    Ok(())
}

#[cfg(unix)]
async fn reload_trigger() {
    use tokio::signal::unix::{SignalKind, signal};
    let sighup = signal(SignalKind::hangup());
    match sighup {
        Ok(mut sighup) => {
            sighup.recv().await;
        }
        _ => {
            future::pending::<Option<()>>().await;
        }
    }
}

#[cfg(not(unix))]
async fn reload_trigger() {
    future::pending().await
}

#[cfg(unix)]
async fn direct_mode_trigger() {
    use tokio::signal::unix::{SignalKind, signal};
    let sigusr1 = signal(SignalKind::user_defined1());
    match sigusr1 {
        Ok(mut sigusr1) => {
            sigusr1.recv().await;
        }
        _ => {
            future::pending::<Option<()>>().await;
        }
    }
}

#[cfg(not(unix))]
async fn direct_mode_trigger() {
    future::pending().await
}

#[cfg(unix)]
async fn shutdown_trigger() {
    use tokio::signal::unix::{SignalKind, signal};
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

fn my_ip_address() -> IpAddr {
    let ipv4 = default_net::get_default_interface()
        .ok()
        .and_then(|i| i.ipv4.first().map(|i| i.addr))
        .unwrap_or_else(|| std::net::Ipv4Addr::new(127, 0, 0, 1));
    IpAddr::from(ipv4)
}
