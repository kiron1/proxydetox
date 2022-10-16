#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod options;

use options::{Authorization, Options};
use proxydetox::auth::netrc;
use proxydetox::auth::AuthenticatorFactory;
use proxydetox::socket;
use std::fs::File;
use std::net::SocketAddr;
use std::result::Result;
use tokio::sync::oneshot;
use tracing_subscriber::filter::EnvFilter;

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
    writeln!(writer, "fatal error: {}", error)?;
    if let Some(cause) = error.source() {
        writeln!(writer, "Caused by:")?;
        for (i, e) in std::iter::successors(Some(cause), |e| e.source()).enumerate() {
            writeln!(writer, "{}: {}", i, e)?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn run(config: &Options) -> Result<(), proxydetox::Error> {
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
        Authorization::Negotiate => unreachable!(),
        Authorization::Basic(netrc_file) => {
            let store = if let Ok(file) = File::open(&netrc_file) {
                netrc::Store::new(std::io::BufReader::new(file))?
            } else {
                netrc::Store::default()
            };
            AuthenticatorFactory::basic(store)
        }
    };

    let session = proxydetox::Session::builder()
        .pac_script(pac_script)
        .authenticator_factory(Some(auth.clone()))
        .always_use_connect(config.always_use_connect)
        .connect_timeout(config.connect_timeout)
        .direct_fallback(config.direct_fallback)
        .build();

    let server = if let Some(name) = &config.activate_socket {
        let sockets = socket::activate_socket(name)?;
        let listener: Vec<std::net::TcpListener> = sockets.take();
        // TODO: currently we only support one listener socket
        let listener = listener
            .into_iter()
            .next()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::Other, "no socket found"))?;
        hyper::Server::from_tcp(listener)?
    } else {
        let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
        hyper::Server::try_bind(&addr)?
    };
    let server = server.serve(session.clone());

    let addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = server.with_graceful_shutdown(async {
        shutdown_rx.await.ok();
    });

    let (timeout_tx, timeout_rx) = oneshot::channel();
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let timeout = config.graceful_shutdown_timeout;
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;
        tokio::spawn(async move {
            tokio::select! {
                _ = sigint.recv() => {},
                _ = sigterm.recv() => {},
            }
            tracing::info!("triggering graceful shutdown");
            shutdown_tx.send(()).ok();
            tokio::time::sleep(timeout).await;
            tracing::info!("graceful shutdown timeout");
            timeout_tx.send(()).ok();
        });
    }
    #[cfg(not(unix))]
    {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("ctrl_c event");
            tracing::info!("triggering graceful shutdown");
            shutdown_tx.send(()).ok();
            tracing::info!("graceful shutdown timeout");
            timeout_tx.send(()).ok();
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
                        session.set_pac_script(pac_script.as_deref()).ok();
                    },
                    _ = sigusr1.recv() => { session.set_pac_script(None).ok();},
                }
            }
        });
    }

    tracing::info!(listening=?addr, authenticator=%auth, pac_file=?config.pac_file, "starting");
    tokio::select! {
        s = server => {
            if let Err(cause) = s {
                tracing::error!(%cause, "fatal server error");
            }
        },
        _ = timeout_rx => {},
    }
    Ok(())
}
