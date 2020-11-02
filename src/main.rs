pub mod auth;
pub mod client;
pub mod detox;
pub mod io;
pub mod net;

#[cfg(target_family = "unix")]
mod limit;

use std::boxed::Box;
use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::result::Result;

use argh::FromArgs;
use hyper::Server;

use crate::detox::DetoxService;

#[derive(Debug, FromArgs)]
/// Proxy tamer
struct Opt {
    /// path to a PAC file
    #[argh(option)]
    pac_file: Option<PathBuf>,

    /// listening port
    #[argh(option, default = "3128")]
    port: u16,
}

fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
    let mut file = File::open(&path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    return Ok(contents);
}

fn load_pac_file(opt: &Opt) -> (Option<PathBuf>, std::io::Result<String>) {
    if let Some(pac_path) = &opt.pac_file {
        return (Some(pac_path.clone()), read_file(pac_path));
    } else {
        let user_config = dirs::config_dir()
            .unwrap_or("".into())
            .join("proxydetox/proxy.pac");
        let config_locations = vec![
            user_config,
            PathBuf::from("/etc/proxydetox/proxy.pac"),
            PathBuf::from("/usr/local/etc/proxydetox/proxy.pac"),
        ];
        for path in config_locations {
            if let Ok(content) = read_file(&path) {
                return (Some(path), Ok(content));
            }
        }
        return (
            None,
            Ok("function FindProxyForURL(url, host) { return \"DIRECT\"; }".into()),
        );
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let opt: Opt = argh::from_env();

    #[cfg(target_family = "unix")]
    limit::update_limits();

    let (pac_path, pac_script) = load_pac_file(&opt);
    if let Some(path) = pac_path {
        tracing::info!("PAC path: {}", path.canonicalize()?.display());
    } else {
        tracing::info!(
            "Using inline PAC config: {}",
            pac_script.as_ref().expect("inline PAC config")
        );
    }

    let pac_script = pac_script.as_ref().expect("inline PAC config");

    loop {
        // Prepare some signal for when the server should start shutting down...
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

        let addr = SocketAddr::from(([127, 0, 0, 1], opt.port));
        let server = Server::bind(&addr).serve(DetoxService::new(&pac_script.clone()));
        let server = server.with_graceful_shutdown(async {
            rx.recv().await.unwrap();
        });

        {
            let mut netrc_path = dirs::home_dir().expect("home");
            netrc_path.push(".netrc");
            let tx = tx.clone();
            tokio::spawn(async move {
                monitor_path(&netrc_path, tx).await;
            });
        }

        tracing::info!("Listening on http://{}", addr);
        if let Err(e) = server.await {
            tracing::error!("server error: {}", e);
            return Err(e.into());
        }
    }
}

async fn monitor_path(path: &Path, mut tx: tokio::sync::mpsc::Sender<()>) {
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
                        tx.send(()).await.unwrap();
                        return;
                    }
                }
            }
        } else {
            interval.tick().await;
        }
    }
}
