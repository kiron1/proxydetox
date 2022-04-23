#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod options;

use options::{Authorization, Options};
use proxydetox::auth::netrc;
use proxydetox::{auth::AuthenticatorFactory, http_file};
use std::fs::{read_to_string, File};
use std::net::SocketAddr;
use std::result::Result;
use tokio::sync::oneshot;
use tracing_subscriber::filter::EnvFilter;

async fn load_pac_file(opt: &Options) -> (Option<String>, std::io::Result<String>) {
    // For Windows, accept a proxy.pac file located next to the binary.
    #[cfg(target_family = "windows")]
    let sys_pac = options::portable_dir("proxy.pac");

    if let Some(pac_path) = &opt.pac_file {
        if pac_path.starts_with("http://") {
            let pac = http_file(pac_path.parse().expect("URI")).await;
            return (Some(pac_path.to_string()), pac);
        }
        (Some(pac_path.to_string()), read_to_string(pac_path))
    } else {
        let user_pac = dirs::config_dir()
            .unwrap_or_else(|| "".into())
            .join("proxydetox/proxy.pac");
        let config_locations = vec![
            user_pac,
            #[cfg(target_family = "unix")]
            std::path::PathBuf::from("/etc/proxydetox/proxy.pac"),
            #[cfg(target_family = "unix")]
            std::path::PathBuf::from("/usr/local/etc/proxydetox/proxy.pac"),
            #[cfg(target_os = "macos")]
            std::path::PathBuf::from("/opt/proxydetox/etc/proxy.pac"),
            #[cfg(target_family = "windows")]
            sys_pac,
        ];
        for path in config_locations {
            if let Ok(content) = read_to_string(&path) {
                return (Some(path.to_string_lossy().to_string()), Ok(content));
            }
        }
        (None, Ok(proxydetox::DEFAULT_PAC_SCRIPT.into()))
    }
}

fn main() {
    let config = Options::load();
    if let Err(cause) = run(&config) {
        tracing::error!(%cause, "fatal error");
        std::process::exit(1);
    }
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

    let (pac_path, pac_script) = load_pac_file(config).await;
    if let Err(cause) = pac_script {
        tracing::error!(%cause, "PAC config error");
        return Err(cause.into());
    }

    let auth = match &config.authorization {
        #[cfg(feature = "negotiate")]
        Authorization::Negotiate => AuthenticatorFactory::negotiate(),
        #[cfg(not(feature = "negotiate"))]
        Authorization::Negotiate => unreachable!(),
        Authorization::Basic(netrc_file) => {
            let store = if let Ok(file) = File::open(&netrc_file) {
                let netrc_store = netrc::Store::new(std::io::BufReader::new(file));
                #[allow(clippy::let_and_return)]
                let netrc_store = match netrc_store {
                    Err(cause) => return Err(cause.into()),
                    Ok(netrc_store) => netrc_store,
                };
                #[cfg(target_os = "linux")]
                {
                    let monitored_netrc_store = netrc_store.clone();
                    let netrc_file = netrc_file.clone();
                    tokio::spawn(async move {
                        monitor_netrc(&netrc_file, monitored_netrc_store).await;
                    });
                }
                netrc_store
            } else {
                netrc::Store::default()
            };
            AuthenticatorFactory::basic(store)
        }
    };

    let session = proxydetox::Session::builder()
        .pac_script(pac_script.ok())
        .authenticator_factory(Some(auth.clone()))
        .always_use_connect(config.always_use_connect)
        .connect_timeout(config.connect_timeout)
        .build();

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let server = hyper::Server::bind(&addr).serve(session);
    let addr = server.local_addr();
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let server = server.with_graceful_shutdown(async {
        shutdown_rx.await.ok();
    });

    #[cfg(unix)]
    {
        use tokio::signal::unix::signal;
        use tokio::signal::unix::SignalKind;
        let mut sigint = signal(SignalKind::interrupt())?;
        let mut sigterm = signal(SignalKind::terminate())?;
        tokio::spawn(async move {
            tokio::select! {
                _ = sigint.recv() => {},
                _ = sigterm.recv() => {},
            }
            tracing::info!("triggering graceful shutdown");
            shutdown_tx.send(()).ok();
        });
    }
    #[cfg(not(unix))]
    {
        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.expect("ctrl_c event");
            tracing::info!("triggering graceful shutdown");
            shutdown_tx.send(()).ok();
        });
    }

    tracing::info!(listening=?addr, authenticator=%auth, pac_file=%pac_path.unwrap_or_default(), "starting");
    if let Err(cause) = server.await {
        tracing::error!("fatal error: {}", cause);
    }
    Ok(())
}

#[cfg(target_os = "linux")]
async fn monitor_netrc(path: impl AsRef<std::path::Path>, store: netrc::Store) {
    use futures_util::StreamExt;
    use inotify::{Inotify, WatchMask};

    fn reload_netrc(path: impl AsRef<std::path::Path>, store: &netrc::Store) {
        tracing::info!(path=%path.as_ref().display(), "change detected");
        if let Ok(file) = File::open(path.as_ref()) {
            if let Err(cause) = store.update(std::io::BufReader::new(file)) {
                tracing::error!("failed to read {}: {}", path.as_ref().display(), cause);
            }
        }
    }

    let parent = path.as_ref().parent().expect("file must have a parent");
    let file_name = path.as_ref().file_name().expect("file must have a name");

    let mut inotify = Inotify::init().expect("Inotify::init");

    inotify
        .add_watch(parent, WatchMask::MOVED_TO | WatchMask::CLOSE_WRITE)
        .expect("add_watch");

    let mut buffer = [0u8; 4096];
    let mut stream = inotify.event_stream(&mut buffer).expect("stream");
    while let Some(event) = stream.next().await {
        match event {
            Ok(ref event) => {
                if event.name.as_ref().map(|n| n == file_name).unwrap_or(false) {
                    reload_netrc(&path, &store);
                }
            }
            Err(ref cause) => {
                tracing::error!("inotify: {}", cause);
                break;
            }
        }
    }
}
