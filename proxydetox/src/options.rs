use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Arg, ArgMatches, Command};
use tracing_subscriber::filter::LevelFilter;

#[derive(Debug, PartialEq)]
pub enum Authorization {
    Basic(PathBuf),
    #[allow(dead_code)]
    Negotiate,
}

#[derive(Debug)]
pub struct Options {
    pub log_level: LevelFilter,
    pub pac_file: Option<String>,
    pub authorization: Authorization,
    pub connect_timeout: std::time::Duration,
    pub always_use_connect: bool,
    pub port: u16,
}

fn is_num<T: FromStr + PartialOrd>(v: &str) -> Result<(), String> {
    match v.parse::<T>() {
        Ok(_v) => Ok(()),
        Err(ref _cause) => Err("invalid number".to_string()),
    }
}

fn is_file(v: &str) -> Result<(), String> {
    if Path::new(&v).is_file() {
        Ok(())
    } else {
        Err(format!("file '{}' does not exists", &v))
    }
}

fn is_file_or_http_uri(v: &str) -> Result<(), String> {
    if v.starts_with("http://") | Path::new(&v).is_file() {
        Ok(())
    } else {
        Err(format!("path '{}' is not a file nor a http URI", &v))
    }
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
        let version = if let Some(hash) = option_env!("PROXYDETOX_BUILD_GIT_HASH") {
            format!("{} ({})", env!("CARGO_PKG_VERSION"), hash)
        } else {
            env!("CARGO_PKG_VERSION").to_owned()
        };
        let app: _ = Command::new(env!("CARGO_PKG_NAME"))
            .version(&*version)
            .about("A small proxy to relive the pain of some corporate proxies")
            .args_override_self(true);

        #[cfg(feature = "negotiate")]
        let app = app.arg(
            Arg::new("negotiate")
                .short('n')
                .long("negotiate")
                .help("Enables Negotiate (SPNEGO) authentication"),
        );

        let netrc_arg = Arg::new("netrc_file")
            .long("netrc-file")
            .help("Path to a .netrc file to be used for basic authentication")
            .validator(is_file)
            .takes_value(true);
        #[cfg(feature = "negotiate")]
        let netrc_arg = netrc_arg.conflicts_with("negotiate");

        let app = app
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .multiple_occurrences(true)
                    .help("Increases verbosity level"),
            )
            .arg(
                 Arg::new("quiet")
                    .short('q')
                    .long("quiet")
                    .multiple_occurrences(true)
                    .help("Decreases verbosity level"),
            )
            .arg(
                Arg::new("port")
                    .short('P')
                    .long("port")
                    .help("Listening port")
                    .validator(is_num::<u16>)
                    .default_value("3128"),
            )
            .arg(
                Arg::new("pac_file")
                    .long("pac-file")
                    .short('p')
                    .help(
                        "PAC file to be used to decide which upstream proxy to forward the request (local file path or http:// URI are accepted)",
                    )
                    .validator(is_file_or_http_uri)
                    .takes_value(true),
            )
            .arg(netrc_arg)
            .arg(
                Arg::new("always_use_connect")
                    .short('C')
                    .long("always-use-connect")
                    .help("Always use CONNECT method even for http:// resources"),
            )
            .arg(
                Arg::new("connect_timeout")
                    .short('c')
                    .long("connect-timeout")
                    .help("Timeout to establish a connection in faction sections")
                    .validator(is_num::<f32>)
                    .takes_value(true)
                    .default_value("10"),
            );

        let matches = app.get_matches_from(args);
        matches.into()
    }
}

impl From<ArgMatches> for Options {
    fn from(m: ArgMatches) -> Self {
        let log_level = 2 /* INFO */;
        let log_level = log_level + m.occurrences_of("verbose") as i32;
        let log_level = log_level - m.occurrences_of("quiet") as i32;
        let log_level = match log_level {
            0 => LevelFilter::ERROR,
            1 => LevelFilter::WARN,
            2 => LevelFilter::INFO,
            3 => LevelFilter::DEBUG,
            4.. => LevelFilter::TRACE,
            _ => LevelFilter::OFF,
        };
        let netrc_file = m
            .value_of("netrc_file")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                let mut netrc_path = dirs::home_dir().unwrap_or_default();
                netrc_path.push(".netrc");
                netrc_path
            });

        #[cfg(feature = "negotiate")]
        let authorization = if m.is_present("negotiate") {
            Authorization::Negotiate
        } else {
            Authorization::Basic(netrc_file)
        };
        #[cfg(not(feature = "negotiate"))]
        let authorization = Authorization::Basic(netrc_file);

        Self {
            log_level,
            pac_file: m.value_of("pac_file").map(String::from),
            authorization,
            always_use_connect: m.is_present("always_use_connect"),
            connect_timeout: m
                .value_of("connect_timeout")
                .map(|s| std::time::Duration::from_secs(s.parse::<u64>().unwrap()))
                .expect("default value for connect_timeout"),
            port: m
                .value_of("port")
                .map(|s| s.parse::<u16>().unwrap())
                .expect("default value for port"),
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
        if let Ok(content) = read_to_string(&path) {
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
    fn test_is_num() {
        assert!(is_num::<u16>("3128").is_ok());
        assert!(is_num::<u16>("hello").is_err());
    }

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
        assert_eq!(args.pac_file, Some(example_pac));
    }

    #[test]
    fn test_pac_file_uri() {
        let proxy_pac = String::from("http://example.org/proxy.pac");
        let args = Options::parse_args(&[
            "proxydetox".into(),
            "--pac-file".into(),
            proxy_pac.clone().into(),
        ]);
        assert_eq!(args.pac_file, Some(proxy_pac));
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
        assert_eq!(args.authorization, Authorization::Negotiate);
    }

    #[test]
    fn test_basic() {
        let args = Options::parse_args(&["proxydetox".into()]);
        assert!(matches!(args.authorization, Authorization::Basic(_)));
    }
}
