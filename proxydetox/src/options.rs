use std::{
    ffi::OsString,
    fs::{File, read_to_string},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use clap::{Arg, ArgAction, ArgMatches, Command};
use detox_net::{PathOrUri, TcpKeepAlive};
use tracing_subscriber::filter::LevelFilter;

lazy_static::lazy_static! {
    static ref NORC: bool = {
        std::env::var(concat!(env!("CARGO_PKG_NAME"), "_NORC").to_uppercase()).map(|s| !s.is_empty()).unwrap_or(false)
    };
}

#[derive(Debug, PartialEq, Eq)]
pub enum Authorization {
    Basic(PathBuf),
    #[allow(dead_code)]
    Negotiate(Vec<String>),
}

#[derive(Debug)]
pub struct Options {
    #[cfg(target_family = "windows")]
    pub attach_console: bool,
    pub log_level: LevelFilter,
    pub log_filepath: Option<PathBuf>,
    pub pac_file: Option<PathOrUri>,
    pub my_ip_address: Option<IpAddr>,
    pub authorization: Authorization,
    pub connect_timeout: Duration,
    pub race_connect: bool,
    pub parallel_connect: usize,
    pub direct_fallback: bool,
    pub proxytunnel: bool,
    pub activate_socket: Option<String>,
    pub listen: Vec<SocketAddr>,
    pub client_tcp_keepalive: TcpKeepAlive,
    #[allow(dead_code)]
    pub server_tcp_keepalive: TcpKeepAlive,
    pub graceful_shutdown_timeout: Duration,
}

fn is_file(v: &str) -> Result<PathBuf, String> {
    let p = Path::new(&v);
    if p.is_file() {
        Ok(p.to_owned())
    } else {
        Err(format!("file '{}' does not exists", &v))
    }
}

fn is_file_or_http_uri(v: &str) -> Result<PathOrUri, String> {
    let p = v.parse::<PathOrUri>().map_err(|e| e.to_string())?;
    if let PathOrUri::Path(ref p) = p {
        if !p.is_file() {
            return Err(format!("path '{}' does not exist or is not a file", &v));
        }
    }
    Ok(p)
}

fn is_valid_socket_addr(v: &str) -> Result<SocketAddr, String> {
    let s = v.parse::<SocketAddr>().map_err(|e| e.to_string())?;
    let unspecified = match s {
        SocketAddr::V4(v4) => v4.ip().is_unspecified(),
        SocketAddr::V6(v6) => v6.ip().is_unspecified(),
    };
    if unspecified {
        return Err(format!("Unspecified addresses is not allowed: {s}"));
    }
    Ok(s)
}

fn which_pac_file() -> Option<PathBuf> {
    // For Windows, accept a proxy.pac file located next to the binary.
    #[cfg(target_family = "windows")]
    let sys_pac = portable_dir("proxy.pac");

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
    config_locations
        .into_iter()
        .find(|path| Path::new(&path).is_file())
}

impl Options {
    #[allow(dead_code)]
    pub fn load() -> Arc<Self> {
        let mut args = Vec::new();
        args.extend(std::env::args_os().take(1));
        if !*NORC {
            args.extend(readrc());
        }
        args.extend(std::env::args_os().skip(1));
        Self::parse_args(&args)
    }

    #[allow(dead_code)]
    pub fn load_without_rcfile() -> Arc<Self> {
        let mut args = Vec::new();
        args.extend(std::env::args_os().take(1));
        args.extend(std::env::args_os().skip(1));
        Self::parse_args(&args)
    }

    pub fn logfile(&self) -> Option<File> {
        self.log_filepath
            .as_ref()
            .map(File::create)
            .transpose()
            .unwrap_or_default()
    }

    fn parse_args(args: &[OsString]) -> Arc<Self> {
        let app = Command::new(env!("CARGO_PKG_NAME"))
            .version(*proxydetoxlib::VERSION_STR)
            .about("A small proxy to relieve the pain of some corporate proxies")
            .args_override_self(true);

        #[cfg(target_family = "windows")]
        let app = app.arg(
            Arg::new("attach_console")
                .long("attach-console")
                .help("Attache to the console of the parent process")
                .action(ArgAction::SetTrue),
        );

        #[cfg(feature = "negotiate")]
        let app = app.arg(
            Arg::new("negotiate")
                .short('n')
                .long("negotiate")
                .value_name("HOST")
                .help("Enables Negotiate (SPNEGO) authentication")
                .action(ArgAction::Append)
                .num_args(0..=1),
        );

        let netrc_arg = Arg::new("netrc_file")
            .long("netrc-file")
            .help("Path to a .netrc file to be used for basic authentication")
            .value_parser(is_file)
            .value_name("PATH")
            .action(clap::ArgAction::Set);
        #[cfg(feature = "negotiate")]
        let netrc_arg = netrc_arg.conflicts_with("negotiate");

        let app = app
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .action(ArgAction::Count)
                    .help("Increases verbosity level"),
            )
            .arg(
                 Arg::new("quiet")
                    .short('q')
                    .long("quiet")
                    .action(ArgAction::Count)
                    .help("Decreases verbosity level"),
            )
            .arg(
                Arg::new("log_filepath")
                    .long("logfile")
                    .value_name("FILEPATH")
                    .help("Log to file instead of stderr")
                    .value_parser(clap::value_parser!(PathBuf))
                    .action(clap::ArgAction::Set)
            )
            .arg(
                 Arg::new("activate_socket")
                     .long("activate-socket")
                     .value_name("NAME")
                     .help("Socket name create by the service manager which needs to be activated")
                     .action(clap::ArgAction::Set),
             )
             .arg(
                Arg::new("listen")
                    .short('L')
                    .long("listen")
                    .value_name("INTERFACE:PORT")
                    .help("Listening interface (e.g. 127.0.0.1:3128)")
                    .value_parser(is_valid_socket_addr)
                    .action(ArgAction::Append)
            )
            .arg(
                Arg::new("pac_file")
                    .long("pac-file")
                    .short('p')
                    .value_name("PATH_OR_URL")
                    .help(
                        "PAC file to be used to decide which upstream proxy to forward the request (local file path, http://, or https:// URI are accepted)",
                    )
                    .value_parser(is_file_or_http_uri)
                    .action(clap::ArgAction::Set),
            )
            .arg(
                Arg::new("my_ip_address")
                    .long("my-ip-address")
                    .value_name("IP-ADDRESS")
                    .help(
                        "Custom IP address to be returned by the myIpAddress PAC function",
                    )
                    .action(clap::ArgAction::Set),
            )
            .arg(netrc_arg)
            .arg(
                Arg::new("proxytunnel")
                    .long("proxytunnel")
                    .help("Always use CONNECT method even for http resources")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("direct_fallback")
                    .long("direct-fallback")
                    .help("Try a direct connection when connecting via proxies fails")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("connect_timeout")
                    .short('c')
                    .long("connect-timeout")
                    .help("Timeout to establish a connection in fraction seconds")
                    .value_name("SECONDS")
                    .value_parser(clap::value_parser!(f64))
                    .action(ArgAction::Set)
                    .default_value("10"),
            )
            .arg(
                Arg::new("race_connect")
                    .long("race-connect")
                    .help("Race multiple connections at the same time")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("parallel_connect")
                    .long("parallel-connect")
                    .help("Number of connect attempts to perform in parallel")
                    .value_name("NUM")
                    .value_parser(clap::value_parser!(usize))
                    .action(ArgAction::Set)
                    .default_value("1"),
            )
            .arg(
                Arg::new("client_tcp_keepalive_time")
                    .long("client-tcp-keepalive-time")
                    .help("TCP keep alive setting for client sockets")
                    .value_name("SECONDS")
                    .value_parser(clap::value_parser!(f64))
                    .action(ArgAction::Set)
            )
            .arg(
                Arg::new("client_tcp_keepalive_interval")
                    .long("client-tcp-keepalive-interval")
                    .help("TCP keep alive setting for client sockets")
                    .value_name("SECONDS")
                    .value_parser(clap::value_parser!(f64))
                    .action(ArgAction::Set)
            )
            .arg(
                Arg::new("client_tcp_keepalive_retries")
                    .long("client-tcp-keepalive-retries")
                    .help("TCP keep alive setting for client sockets")
                    .value_name("COUNT")
                    .value_parser(clap::value_parser!(u32))
                    .action(ArgAction::Set)
            )  .arg(
                Arg::new("server_tcp_keepalive_time")
                    .long("server-tcp-keepalive-time")
                    .help("TCP keep alive setting for server sockets")
                    .value_name("SECONDS")
                    .value_parser(clap::value_parser!(f64))
                    .action(ArgAction::Set)
            )
            .arg(
                Arg::new("server_tcp_keepalive_interval")
                    .long("server-tcp-keepalive-interval")
                    .help("TCP keep alive setting for server sockets")
                    .value_name("SECONDS")
                    .value_parser(clap::value_parser!(f64))
                    .action(ArgAction::Set)
            )
            .arg(
                Arg::new("server_tcp_keepalive_retries")
                    .long("server-tcp-keepalive-retries")
                    .help("TCP keep alive setting for server sockets")
                    .value_name("COUNT")
                    .value_parser(clap::value_parser!(u32))
                    .action(ArgAction::Set)
            )
            .arg(
                Arg::new("graceful_shutdown_timeout")
                    .long("graceful-shutdown-timeout")
                    .help("Timeout to wait for a graceful shutdown")
                    .default_value("30")
                    .value_parser(clap::value_parser!(u64))
                    .action(ArgAction::Set)
                    .value_name("SECONDS"),
            );

        let matches = app.get_matches_from(args);
        Arc::new(matches.into())
    }
}

impl From<ArgMatches> for Options {
    fn from(m: ArgMatches) -> Self {
        let log_level = 2 /* INFO */;
        let log_level = log_level + m.get_count("verbose") as i32;
        let log_level = log_level - m.get_count("quiet") as i32;
        let log_level = match log_level {
            0 => LevelFilter::ERROR,
            1 => LevelFilter::WARN,
            2 => LevelFilter::INFO,
            3 => LevelFilter::DEBUG,
            4.. => LevelFilter::TRACE,
            _ => LevelFilter::OFF,
        };
        let netrc_file = m
            .get_one::<PathBuf>("netrc_file")
            .cloned()
            .unwrap_or_else(|| {
                let mut netrc_path = dirs::home_dir().unwrap_or_default();
                netrc_path.push(".netrc");
                netrc_path
            });

        #[cfg(feature = "negotiate")]
        let authorization = match m.get_many::<String>("negotiate") {
            Some(negotiate) => Authorization::Negotiate(negotiate.cloned().collect()),
            _ => Authorization::Basic(netrc_file),
        };
        #[cfg(not(feature = "negotiate"))]
        let authorization = Authorization::Basic(netrc_file);

        let listen = match m.get_many::<SocketAddr>("listen") {
            Some(listen) => listen.cloned().collect(),
            _ => {
                vec![SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
                    3128,
                )]
            }
        };

        let client_tcp_keepalive = TcpKeepAlive::new()
            .with_time(
                m.get_one::<f64>("client_tcp_keepalive_time")
                    .map(|s| Duration::from_millis((*s * 1000.0) as u64)),
            )
            .with_interval(
                m.get_one::<f64>("client_tcp_keepalive_interval")
                    .map(|s| Duration::from_millis((*s * 1000.0) as u64)),
            )
            .with_retries(m.get_one::<u32>("client_tcp_keepalive_retries").cloned());
        let server_tcp_keepalive = TcpKeepAlive::new()
            .with_time(
                m.get_one::<f64>("server_tcp_keepalive_time")
                    .map(|s| Duration::from_millis((*s * 1000.0) as u64)),
            )
            .with_interval(
                m.get_one::<f64>("server_tcp_keepalive_interval")
                    .map(|s| Duration::from_millis((*s * 1000.0) as u64)),
            )
            .with_retries(m.get_one::<u32>("server_tcp_keepalive_retries").cloned());

        Self {
            #[cfg(target_family = "windows")]
            attach_console: m.get_flag("attach_console"),
            log_level,
            log_filepath: m.get_one("log_filepath").cloned(),
            pac_file: m
                .get_one::<PathOrUri>("pac_file")
                .cloned()
                .or_else(|| which_pac_file().map(PathOrUri::Path)),
            my_ip_address: m.get_one::<IpAddr>("my_ip_address").cloned(),
            authorization,
            proxytunnel: m.get_flag("proxytunnel"),
            direct_fallback: m.get_flag("direct_fallback"),
            connect_timeout: m
                .get_one::<f64>("connect_timeout")
                .map(|s| Duration::from_millis((*s * 1000.0) as u64))
                .expect("default value for connect_timeout"),
            race_connect: m.get_flag("race_connect"),
            parallel_connect: m
                .get_one::<usize>("parallel_connect")
                .copied()
                .expect("default value for parallel_connect"),
            activate_socket: m.get_one::<String>("activate_socket").cloned(),
            listen,
            client_tcp_keepalive,
            server_tcp_keepalive,
            graceful_shutdown_timeout: m
                .get_one::<u64>("graceful_shutdown_timeout")
                .map(|s| Duration::from_secs(*s))
                .expect("default value for graceful_shutdown_timeout"),
        }
    }
}

/// Load config file, but command line flags will override config file values.
#[allow(dead_code)]
fn readrc() -> Vec<OsString> {
    let user_config = dirs::config_dir()
        .unwrap_or_else(|| "".into())
        .join("proxydetox/proxydetoxrc");
    let config_locations = vec![
        user_config,
        #[cfg(target_family = "unix")]
        PathBuf::from("/etc/proxydetox/proxydetoxrc"),
        #[cfg(target_family = "unix")]
        PathBuf::from("/usr/local/etc/proxydetox/proxydetoxrc"),
        #[cfg(target_os = "macos")]
        std::path::PathBuf::from("/opt/proxydetox/etc/proxydetoxrc"),
        #[cfg(target_family = "windows")]
        portable_dir("proxydetoxrc"),
        #[cfg(target_family = "windows")]
        portable_dir("proxydetoxrc.txt"),
    ];
    for path in config_locations {
        if let Ok(content) = read_to_string(path) {
            // todo: this will fail with arguments which require a space (e.g. path of pac_file)
            let args = content
                .split('\n')
                .map(|s| s.trim())
                .filter(|s| !s.starts_with('#'))
                .flat_map(str::split_ascii_whitespace)
                .filter(|s| !s.is_empty())
                .map(OsString::from)
                .collect::<Vec<_>>();
            return args;
        }
    }
    Vec::new()
}

#[cfg(target_family = "windows")]
// For Windows, use config file path next to the binary for portable use cases.
pub fn portable_dir(path: impl AsRef<Path>) -> PathBuf {
    let sys_config = std::env::current_exe()
        .map(|p| {
            p.parent()
                .map(|p| {
                    let mut p = PathBuf::from(p);
                    p.push(path);
                    p
                })
                .unwrap_or_default()
        })
        .unwrap_or_default();
    sys_config
}

#[cfg(test)]
mod tests {
    use super::*;

    fn example_pac() -> String {
        let mut p = if let Ok(runfiles_dir) = std::env::var("RUNFILES_DIR") {
            let mut p = PathBuf::from(runfiles_dir);
            if let Ok(workspace) = std::env::var("TEST_WORKSPACE") {
                p.push(workspace);
            } else {
                p.push("proxydetox"); // Bazel workspace name
            }
            p
        } else {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .to_owned()
        };
        p.push("paceval");
        p.push("example.pac");
        p.into_os_string().into_string().expect("example.pac")
    }

    #[test]
    fn test_is_file() {
        let pac_file = example_pac();
        assert!(is_file(&pac_file).is_ok());
        assert!(is_file("/does/not/exist").is_err());
    }

    #[test]
    fn test_is_file_or_uri() {
        assert!(is_file_or_http_uri("http://example.org/").is_ok());
        assert!(is_file_or_http_uri("https://example.org/").is_ok());
        assert!(is_file_or_http_uri("/does/not/exist").is_err());
    }

    #[test]
    fn test_default() {
        let args = Options::parse_args(&["proxydetox".into()]);
        assert!(!args.direct_fallback);
        assert!(!args.proxytunnel);
    }

    #[test]
    fn test_not_default() {
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--direct-fallback".into(),
            "--proxytunnel".into(),
        ]);
        assert!(args.direct_fallback);
        assert!(args.proxytunnel);
    }

    #[test]
    fn test_pac_file_path() {
        let pac_file = example_pac();
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--pac-file".into(),
            pac_file.clone().into(),
        ]);
        assert_eq!(
            args.pac_file,
            Some(PathOrUri::Path(PathBuf::from(&pac_file)))
        );
    }

    #[test]
    fn test_pac_file_uri() {
        let proxy_pac = String::from("http://example.org/proxy.pac");
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--pac-file".into(),
            proxy_pac.clone().into(),
        ]);
        assert_eq!(args.pac_file, proxy_pac.parse().ok());
    }

    #[test]
    fn test_listen_none() {
        let args = Options::parse_args(&["proxydetox".into()]);
        assert_eq!(args.listen, vec!["127.0.0.1:3128".parse().unwrap()]);
    }

    #[test]
    fn test_listen_one() {
        let addr_str = "192.168.0.1:8080";
        let addr = addr_str.parse::<SocketAddr>().unwrap();
        let args = Options::parse_args(&["proxydetox".into(), "--listen".into(), addr_str.into()]);
        assert_eq!(args.listen, vec![addr]);
    }

    #[test]
    fn test_listen_many() {
        let addr1_str = "192.168.0.1:8080";
        let addr2_str = "10.0.0.1:3128";
        let addr1 = addr1_str.parse::<SocketAddr>().unwrap();
        let addr2 = addr2_str.parse::<SocketAddr>().unwrap();
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--listen".into(),
            addr1_str.into(),
            "--listen".into(),
            addr2_str.into(),
        ]);
        assert_eq!(args.listen, vec![addr1, addr2]);
    }

    #[cfg(feature = "negotiate")]
    #[test]
    fn test_negotiate() {
        let args = Options::parse_args(&["proxydetox".into(), "--negotiate".into()]);
        assert!(matches!(args.authorization, Authorization::Negotiate(_)));
    }

    #[cfg(feature = "negotiate")]
    #[test]
    fn test_negotiate_host() {
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--negotiate".into(),
            "proxyA.exampe.net".into(),
            "--negotiate".into(),
            "proxyB.exampe.net".into(),
        ]);
        assert_eq!(
            args.authorization,
            Authorization::Negotiate(vec!["proxyA.exampe.net".into(), "proxyB.exampe.net".into(),])
        );
    }

    #[test]
    fn test_basic() {
        let args = Options::parse_args(&["proxydetox".into()]);
        assert!(matches!(args.authorization, Authorization::Basic(_)));
    }

    #[test]
    fn test_tcp_keep_alive() {
        let args = &[
            "proxydetox",
            "--client-tcp-keepalive-time",
            "10.0",
            "--client-tcp-keepalive-interval",
            "20.0",
            "--client-tcp-keepalive-retries",
            "5",
            "--server-tcp-keepalive-time",
            "100.0",
            "--server-tcp-keepalive-interval",
            "200.0",
            "--server-tcp-keepalive-retries",
            "50",
        ]
        .iter()
        .map(OsString::from)
        .collect::<Vec<_>>();
        let args = Options::parse_args(args);

        assert_eq!(
            args.client_tcp_keepalive.time(),
            Some(Duration::from_secs(10))
        );
        assert_eq!(
            args.client_tcp_keepalive.interval(),
            Some(Duration::from_secs(20))
        );
        assert_eq!(args.client_tcp_keepalive.retries(), Some(5));

        assert_eq!(
            args.server_tcp_keepalive.time(),
            Some(Duration::from_secs(100))
        );
        assert_eq!(
            args.server_tcp_keepalive.interval(),
            Some(Duration::from_secs(200))
        );
        assert_eq!(args.server_tcp_keepalive.retries(), Some(50));
    }
}
