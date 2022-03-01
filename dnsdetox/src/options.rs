use http::Uri;
use std::{ffi::OsString, fs::read_to_string, net::SocketAddr, path::PathBuf, str::FromStr};

use clap::{Arg, ArgMatches, Command};

#[derive(Debug)]
pub struct Options {
    pub port: u16,
    pub proxy: Uri,
    pub primary: SocketAddr,
    pub secondary: Uri,
}

fn is_num<T: FromStr + PartialOrd>(v: &str) -> Result<(), String> {
    match v.parse::<T>() {
        Ok(_v) => Ok(()),
        Err(ref _cause) => Err("invalid number".to_string()),
    }
}

impl Options {
    pub fn load() -> Self {
        let app: _ = Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about("A small DNS proxy to relive the pain of some corporate networks")
            .args_override_self(true);

        let app = app
            .arg(
                Arg::new("port")
                    .long("port")
                    .value_name("PORT")
                    .help("Listening port")
                    .validator(is_num::<u16>)
                    .default_value("5353"),
            )
            .arg(
                Arg::new("proxy")
                    .long("proxy")
                    .value_name("PROXY")
                    .env("http_proxy")
                    .help("Internet proxy")
                    .default_value("http://127.0.0.1:3128"),
            )
            .arg(
                Arg::new("primary")
                    .long("primary")
                    .value_name("IP:PORT")
                    .help("Primary DNS server using UDP protocol")
                    .required(true)
                    .takes_value(true),
            )
            .arg(
                Arg::new("secondary")
                    .long("secondary")
                    .value_name("URI")
                    .help("Secondary DNS server using DNS over HTTPS (DoH) protocol")
                    .default_value("https://8.8.8.8/dns-query")
                    .required(true)
                    .takes_value(true),
            );

        let mut args = Vec::new();
        args.extend(std::env::args_os().take(1));
        args.extend(readrc());
        args.extend(std::env::args_os().skip(1));
        let matches = app.get_matches_from(args);

        matches.into()
    }
}

impl From<ArgMatches> for Options {
    fn from(m: ArgMatches) -> Self {
        Self {
            port: m
                .value_of("port")
                .map(|s| s.parse::<u16>().unwrap())
                .unwrap(),
            proxy: m
                .value_of("proxy")
                .map(|s| s.parse::<Uri>().unwrap())
                .unwrap(),
            primary: m
                .value_of("primary")
                .map(|s| s.parse::<SocketAddr>().unwrap())
                .unwrap(),
            secondary: m
                .value_of("secondary")
                .map(|s| s.parse::<Uri>().unwrap())
                .unwrap(),
        }
    }
}

/// Load config file, but command line flags will override config file values.
fn readrc() -> Vec<OsString> {
    let user_config = dirs::config_dir()
        .unwrap_or_else(|| "".into())
        .join("dnsdetox/dnsdetoxrc");
    let config_locations = vec![
        user_config,
        #[cfg(target_family = "unix")]
        PathBuf::from("/etc/dnsdetox/dnsdetoxrc"),
        #[cfg(target_family = "unix")]
        PathBuf::from("/usr/local/etc/dnsdetox/dnsdetoxrc"),
        #[cfg(target_family = "windows")]
        portable_dir("dnsdetoxrc"),
        #[cfg(target_family = "windows")]
        portable_dir("dnsdetoxrc.txt"),
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
pub fn portable_dir(path: impl AsRef<std::path::Path>) -> PathBuf {
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
