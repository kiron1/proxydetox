#[cfg(target_family = "unix")]
mod limit;

use std::fmt::Display;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::result::Result;
use std::{boxed::Box, str::FromStr};

use argh::FromArgs;
use hyper::Server;
use proxydetox::auth::AuthenticatorFactory;
use proxydetox::detox;
use proxydetox::http_file;
use proxydetox::read_file;

#[derive(Debug, FromArgs)]
/// Proxy tamer
struct Options {
    /// use HTTP Negotiate instead of netrc to authenticate against proxies
    #[cfg(feature = "gssapi")]
    #[argh(switch)]
    negotiate: bool,

    /// path to a PAC file or url of PAC file
    #[argh(option)]
    pac_file: Option<String>,

    /// listening port
    #[argh(option)]
    port: Option<u16>,

    /// sets the maximum idle connection per host allowed in the pool
    #[argh(option, default = "usize::MAX")]
    pool_max_idle_per_host: usize,

    /// set an optional timeout for idle sockets being kept-aliv.
    #[argh(option)]
    pool_idle_timeout: Option<Seconds>,
}

#[derive(Copy, Clone, Debug)]
struct Seconds(u64);

impl Display for Seconds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}s", self.0)
    }
}

impl FromStr for Seconds {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let n = s.parse()?;
        Ok(Seconds(n))
    }
}

impl From<Seconds> for std::time::Duration {
    fn from(sec: Seconds) -> Self {
        std::time::Duration::from_secs(sec.0)
    }
}

/// Load config file, but command line flags will override config file values.
fn load_config() -> Options {
    let opt: Options = argh::from_env();
    let user_config = dirs::config_dir()
        .unwrap_or_else(|| "".into())
        .join("proxydetox/proxydetoxrc");
    let config_locations = vec![
        user_config,
        PathBuf::from("/etc/proxydetox/proxydetoxrc"),
        PathBuf::from("/usr/local/etc/proxydetox/proxydetoxrc"),
    ];
    for path in config_locations {
        if let Ok(content) = read_file(&path) {
            let name = std::env::args().next().expect("argv[0]");
            // todo: this will fail with arguments which require a space (e.g. path of pac_file)
            let args = content
                .split('\n')
                .map(|s| s.trim())
                .filter(|s| !s.starts_with('#'))
                .map(str::split_ascii_whitespace)
                .flatten()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>();
            let rcopt = Options::from_args(&[&name], &args).expect("valid proxydetoxrc file");
            // Return merged options, priotize command line flags over file.
            return Options {
                #[cfg(feature = "gssapi")]
                negotiate: if opt.negotiate { true } else { rcopt.negotiate },
                pac_file: opt.pac_file.or(rcopt.pac_file),
                port: opt.port.or(rcopt.port),
                pool_max_idle_per_host: opt
                    .pool_max_idle_per_host
                    .min(rcopt.pool_max_idle_per_host),
                pool_idle_timeout: opt.pool_idle_timeout.or(rcopt.pool_idle_timeout),
            };
        }
    }
    opt
}

fn load_pac_file(opt: &Options) -> (Option<String>, std::io::Result<String>) {
    if let Some(pac_path) = &opt.pac_file {
        if pac_path.starts_with("http://") {
            let pac = futures::executor::block_on(async {
                http_file(pac_path.parse().expect("URI")).await
            });
            return (Some(pac_path.to_string()), pac);
        }
        (Some(pac_path.to_string()), read_file(pac_path))
    } else {
        let user_config = dirs::config_dir()
            .unwrap_or_else(|| "".into())
            .join("proxydetox/proxy.pac");
        let config_locations = vec![
            user_config,
            PathBuf::from("/etc/proxydetox/proxy.pac"),
            PathBuf::from("/usr/local/etc/proxydetox/proxy.pac"),
        ];
        for path in config_locations {
            if let Ok(content) = read_file(&path) {
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

    let config = load_config();

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

    loop {
        // Prepare some signal for when the server should start shutting down...
        let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(32);

        #[cfg(feature = "gssapi")]
        let auth = if config.negotiate {
            AuthenticatorFactory::negotiate()
        } else {
            AuthenticatorFactory::basic()
        };

        #[cfg(not(feature = "gssapi"))]
        let auth = AuthenticatorFactory::basic();

        tracing::info!("Authenticator factory: {}", &auth);

        let detox_config = detox::Config {
            pool_idle_timeout: config.pool_idle_timeout.map(|x| x.into()),
            pool_max_idle_per_host: config.pool_max_idle_per_host,
        };

        let addr = SocketAddr::from(([127, 0, 0, 1], config.port.unwrap_or(3128)));
        let server =
            Server::bind(&addr).serve(detox::Service::new(&pac_script.clone(), auth, detox_config));
        let server = server.with_graceful_shutdown(async {
            rx.recv().await.unwrap();
        });

        {
            use tokio::signal;
            let tx = tx.clone();
            tokio::spawn(async move {
                signal::ctrl_c().await.expect("ctrl_c event");
                tx.send(()).await.unwrap();
            });
        }

        #[cfg(target_family = "unix")]
        {
            use tokio::signal::unix::{signal, SignalKind};
            let tx = tx.clone();
            let mut stream = signal(SignalKind::hangup())?;
            tokio::spawn(async move {
                stream.recv().await;
                tx.send(()).await.unwrap();
            });
        }

        #[cfg(target_os = "linux")]
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

#[cfg(target_os = "linux")]
async fn monitor_path(path: &Path, tx: tokio::sync::mpsc::Sender<()>) {
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
