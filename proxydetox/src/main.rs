#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(target_family = "unix")]
mod limit;
mod options;

use options::Options;
use std::boxed::Box;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::result::Result;

use proxydetox::{auth::AuthenticatorFactory, detox, http_file};

fn load_pac_file(opt: &Options) -> (Option<String>, std::io::Result<String>) {
    if let Some(pac_path) = &opt.pac_file {
        if pac_path.starts_with("http://") {
            let pac = futures::executor::block_on(async {
                http_file(pac_path.parse().expect("URI")).await
            });
            return (Some(pac_path.to_string()), pac);
        }
        (Some(pac_path.to_string()), read_to_string(pac_path))
    } else {
        let user_config = dirs::config_dir()
            .unwrap_or_else(|| "".into())
            .join("proxydetox/proxy.pac");
        let config_locations = vec![
            user_config,
            #[cfg(target_family = "unix")]
            PathBuf::from("/etc/proxydetox/proxy.pac"),
            #[cfg(target_family = "unix")]
            PathBuf::from("/usr/local/etc/proxydetox/proxy.pac"),
        ];
        for path in config_locations {
            if let Ok(content) = read_to_string(&path) {
                return (Some(path.to_string_lossy().to_string()), Ok(content));
            }
        }
        (
            None,
            Ok("function FindProxyForURL(url, host) { return \"DIRECT\"; }".into()),
        )
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = Options::load();

    #[cfg(target_family = "unix")]
    limit::update_limits();

    let (pac_path, pac_script) = load_pac_file(&config);
    if let Some(path) = pac_path {
        tracing::info!("PAC path: {}", &path);
    } else {
        tracing::info!(
            "Using inline PAC config: {}",
            pac_script.as_ref().expect("inline PAC config")
        );
    }

    let pac_script = pac_script.as_ref().expect("inline PAC config");

    #[cfg(feature = "negotiate")]
    let auth = if config.negotiate {
        AuthenticatorFactory::negotiate()
    } else {
        AuthenticatorFactory::basic(config.netrc_file.clone())
    };

    #[cfg(not(feature = "negotiate"))]
    let auth = AuthenticatorFactory::basic(config.netrc_file.clone());

    tracing::info!("Authenticator factory: {}", &auth);

    let detox_config = detox::Config {
        pool_idle_timeout: config.pool_idle_timeout,
        pool_max_idle_per_host: config.pool_max_idle_per_host,
        always_use_connect: config.always_use_connect,
    };

    let mut server = proxydetox::Server::new(pac_script.clone(), auth, config.port, detox_config);

    {
        use tokio::signal;
        let tx = server.control_channel();
        tokio::spawn(async move {
            loop {
                signal::ctrl_c().await.expect("ctrl_c event");
                tracing::info!("received Ctrl-C, trigger shutdown");
                let _ = tx.send(proxydetox::Command::Shutdown).await;
            }
        });
    }

    #[cfg(target_family = "unix")]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let tx = server.control_channel();
        let mut stream = signal(SignalKind::hangup())?;
        tokio::spawn(async move {
            loop {
                stream.recv().await;
                tracing::info!("received SIGHUP, trigger restart");
                let _ = tx.send(proxydetox::Command::Restart).await;
            }
        });
    }

    #[cfg(target_os = "linux")]
    {
        let netrc_path = config.netrc_file.clone();
        let tx = server.control_channel();
        tokio::spawn(async move {
            monitor_path(&netrc_path, tx).await;
        });
    }

    server.run().await
}

#[cfg(target_os = "linux")]
async fn monitor_path(path: &std::path::Path, tx: tokio::sync::mpsc::Sender<proxydetox::Command>) {
    use futures_util::StreamExt;
    use inotify::{EventMask, Inotify, WatchMask};
    use tokio::time::{self, Duration};

    let mut interval = time::interval(Duration::from_secs(3));

    let mut inotify = Inotify::init().expect("Inotify::init");

    loop {
        if path.exists() {
            inotify
                .add_watch(path, WatchMask::DELETE_SELF | WatchMask::MODIFY)
                .expect("add_watch");
            let mut buffer = [0; 32];
            let mut stream = inotify.event_stream(&mut buffer).expect("stream");
            while let Some(event_or_error) = stream.next().await {
                if let Ok(event) = event_or_error {
                    if event.mask.contains(EventMask::DELETE_SELF)
                        || event.mask.contains(EventMask::MODIFY)
                    {
                        tracing::info!("~/.netrc changed, trigger restart");
                        let _ = tx.send(proxydetox::Command::Restart).await;
                        break;
                    }
                }
            }
        } else {
            interval.tick().await;
        }
    }
}
