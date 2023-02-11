use std::{
    ffi::OsString,
    fs::read_to_string,
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use clap::{Arg, ArgAction, ArgMatches, Command};
use http::Uri;
use tracing_subscriber::filter::LevelFilter;

lazy_static::lazy_static! {
    static ref VERSION: String = {
        if let Some(hash) = option_env!("PROXYDETOX_BUILD_GIT_HASH") {
            format!("{} ({})", env!("CARGO_PKG_VERSION"), hash)
        } else {
            env!("CARGO_PKG_VERSION").to_owned()
        }
    };

    static ref VERSION_STR: &'static str = &VERSION;
}
#[derive(Debug, PartialEq, Eq)]
pub enum Authorization {
    Basic(PathBuf),
    #[allow(dead_code)]
    Negotiate(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathOrUri {
    Path(PathBuf),
    Uri(Uri),
}

impl PathOrUri {
    pub async fn contents(&self) -> std::io::Result<String> {
        match *self {
            PathOrUri::Path(ref p) => read_to_string(p),
            PathOrUri::Uri(ref u) => proxydetoxlib::http_file(u.clone()).await,
        }
    }
}

impl FromStr for PathOrUri {
    type Err = http::uri::InvalidUri;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("http://") {
            Ok(Self::from(s.parse::<Uri>()?))
        } else {
            Ok(Self::from(PathBuf::from(s)))
        }
    }
}

impl From<PathBuf> for PathOrUri {
    fn from(path: PathBuf) -> Self {
        Self::Path(path)
    }
}

impl From<Uri> for PathOrUri {
    fn from(uri: Uri) -> Self {
        Self::Uri(uri)
    }
}

#[derive(Debug)]
pub struct Options {
    pub log_level: LevelFilter,
    pub pac_file: Option<PathOrUri>,
    pub authorization: Authorization,
    pub connect_timeout: Duration,
    pub direct_fallback: bool,
    pub always_use_connect: bool,
    pub activate_socket: Option<String>,
    pub interface: IpAddr,
    pub port: u16,
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
    if v.starts_with("http://") & v.parse::<Uri>().is_ok() {
        Ok(PathOrUri::Uri(v.parse::<Uri>().unwrap()))
    } else if Path::new(&v).is_file() {
        Ok(PathOrUri::Path(PathBuf::from(v)))
    } else {
        Err(format!("path '{}' is not a file nor a http URI", &v))
    }
}

fn is_ip(v: &str) -> Result<IpAddr, String> {
    match v.parse::<IpAddr>() {
        Ok(ip) => Ok(ip),
        Err(_) => Err(format!("value '{}' is not a valid IP address", &v)),
    }
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
    pub fn load() -> Self {
        let mut args = Vec::new();
        args.extend(std::env::args_os().take(1));
        args.extend(readrc());
        args.extend(std::env::args_os().skip(1));
        Self::parse_args(&args)
    }

    fn parse_args(args: &[OsString]) -> Self {
        let app = Command::new(env!("CARGO_PKG_NAME"))
            .version(*VERSION_STR)
            .about("A small proxy to relieve the pain of some corporate proxies")
            .args_override_self(true);

        #[cfg(feature = "negotiate")]
        let app = app.arg(
            Arg::new("negotiate")
                .short('n')
                .long("negotiate")
                .help("Enables Negotiate (SPNEGO) authentication")
                .action(ArgAction::Append)
                .num_args(0..=1),
        );

        let netrc_arg = Arg::new("netrc_file")
            .long("netrc-file")
            .help("Path to a .netrc file to be used for basic authentication")
            .value_parser(is_file)
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
                 Arg::new("activate_socket")
                     .long("activate-socket")
                     .help("Socket name create by the service manager which needs to be activated")
                     .action(clap::ArgAction::Set),
             )
             .arg(
                Arg::new("interface")
                    .long("interface")
                    .short('i')
                    .help("Interface to listen on for incoming connections")
                    .default_value("127.0.0.1")
                    .value_parser(is_ip)
                    .action(ArgAction::Set),
            )
            .arg(
                Arg::new("port")
                    .short('P')
                    .long("port")
                    .help("Listening port")
                    .value_parser(clap::value_parser!(u16))
                    .action(ArgAction::Set)
                    .default_value("3128"),
            )
            .arg(
                Arg::new("pac_file")
                    .long("pac-file")
                    .short('p')
                    .help(
                        "PAC file to be used to decide which upstream proxy to forward the request (local file path or http:// URI are accepted)",
                    )
                    .value_parser(is_file_or_http_uri)
                    .action(clap::ArgAction::Set),
            )
            .arg(netrc_arg)
            .arg(
                Arg::new("always_use_connect")
                    .short('C')
                    .long("always-use-connect")
                    .help("Always use CONNECT method even for http:// resources")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("direct_fallback")
                    .long("direct-fallback")
                    .help("Try a direct connection when connecting proxies fails")
                    .action(ArgAction::SetTrue),
            )
            .arg(
                Arg::new("connect_timeout")
                    .short('c')
                    .long("connect-timeout")
                    .help("Timeout to establish a connection in faction sections")
                    .value_parser(clap::value_parser!(f64))
                    .action(ArgAction::Set)
                    .default_value("10"),
            )
            .arg(
                Arg::new("graceful_shutdown_timeout")
                    .long("graceful-shutdown-timeout")
                    .help("Timeout to wait for a graceful shutdown")
                    .value_parser(clap::value_parser!(u64))
                    .action(ArgAction::Set)
                    .default_value("30"),
            );

        let matches = app.get_matches_from(args);
        matches.into()
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
        let authorization = if let Some(negotiate) = m.get_many::<String>("negotiate") {
            Authorization::Negotiate(negotiate.cloned().collect())
        } else {
            Authorization::Basic(netrc_file)
        };
        #[cfg(not(feature = "negotiate"))]
        let authorization = Authorization::Basic(netrc_file);

        Self {
            log_level,
            pac_file: m
                .get_one::<PathOrUri>("pac_file")
                .cloned()
                .or_else(|| which_pac_file().map(PathOrUri::from)),
            authorization,
            always_use_connect: m.contains_id("always_use_connect"),
            direct_fallback: m.contains_id("direct_fallback"),
            connect_timeout: m
                .get_one::<f64>("connect_timeout")
                .map(|s| Duration::from_millis((*s * 1000.0) as u64))
                .expect("default value for connect_timeout"),
            activate_socket: m.get_one::<String>("activate_socket").cloned(),
            interface: *m.get_one::<IpAddr>("interface").unwrap(),
            port: *m.get_one::<u16>("port").expect("default value for port"),
            graceful_shutdown_timeout: m
                .get_one::<u64>("graceful_shutdown_timeout")
                .map(|s| Duration::from_secs(*s))
                .expect("default value for graceful_shutdown_timeout"),
        }
    }
}

/// Load config file, but command line flags will override config file values.
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

    #[test]
    fn test_is_file() {
        let mut example_pac = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned();
        example_pac.push("paceval");
        example_pac.push("example.pac");
        let example_pac = example_pac.to_str().unwrap().to_owned();

        assert!(is_file(&example_pac).is_ok());
        assert!(is_file("/does/not/exist").is_err());
    }

    #[test]
    fn test_is_ip() {
        assert!(is_ip("0.0.0.0").is_ok());
        assert!(is_ip("::1").is_ok());
        assert!(is_ip("this.is.not.ip").is_err())
    }

    #[test]
    fn test_is_file_or_uri() {
        assert!(is_file_or_http_uri("http://example.org/").is_ok());
        assert!(is_file_or_http_uri("/does/not/exist").is_err());
    }

    #[test]
    fn test_pac_file_path() {
        let mut example_pac = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_owned();
        example_pac.push("paceval");
        example_pac.push("example.pac");
        let example_pac = example_pac.to_str().unwrap().to_owned();
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--pac-file".into(),
            example_pac.clone().into(),
        ]);
        assert_eq!(args.pac_file, example_pac.parse().ok());
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
    fn test_interface() {
        let addr = String::from("0.0.0.0");
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--interface".into(),
            addr.clone().into(),
        ]);
        assert_eq!(args.interface, addr.parse::<IpAddr>().unwrap());
    }

    #[test]
    fn test_port() {
        let args = Options::parse_args(&["proxydetox".into(), "--port".into(), "8080".into()]);
        assert_eq!(args.port, 8080);
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
}
