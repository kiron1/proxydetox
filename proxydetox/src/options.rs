use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use clap::{Arg, ArgMatches, Command};

#[derive(Debug)]
pub struct Options {
    #[cfg(feature = "negotiate")]
    pub negotiate: bool,
    pub pac_file: Option<String>,
    pub netrc_file: PathBuf,
    pub always_use_connect: bool,
    pub port: u16,
    pub pool_max_idle_per_host: usize,
    pub pool_idle_timeout: Option<Duration>,
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

impl Options {
    pub fn load() -> Self {
        let default_pool_max_idle_per_host = usize::MAX.to_string();

        let app: _ = Command::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
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
                        "PAC file to be used to decide which upstream proxy to forward the request",
                    )
                    .validator(is_file)
                    .takes_value(true),
            )
            .arg(netrc_arg)
            .arg(
                Arg::new("always_use_connect")
                    .short('c')
                    .long("always_use_connect")
                    .help("Always use CONNECT method even for http:// resources"),
            )
            .arg(
                Arg::new("pool_max_idle_per_host")
                    .long("pool-max-idle-per-host")
                    .help("Maximum idle connection per host allowed in the pool")
                    .validator(is_num::<usize>)
                    .default_value(&default_pool_max_idle_per_host),
            )
            .arg(
                Arg::new("pool_idle_timeout")
                    .long("pool-idle-timeout")
                    .help("Optional timeout for idle sockets being kept-alive")
                    .validator(is_num::<u64>)
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
            #[cfg(feature = "negotiate")]
            negotiate: m.is_present("negotiate"),
            pac_file: m.value_of("pac_file").map(String::from),
            netrc_file: m
                .value_of("netrc_file")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    let mut netrc_path = dirs::home_dir().unwrap_or_default();
                    netrc_path.push(".netrc");
                    netrc_path
                }),
            always_use_connect: m.is_present("always_use_connect"),
            port: m
                .value_of("port")
                .map(|s| s.parse::<u16>().unwrap())
                .unwrap(),
            pool_max_idle_per_host: m
                .value_of("pool_max_idle_per_host")
                .map(|s| s.parse::<usize>().unwrap())
                .unwrap(),
            pool_idle_timeout: m
                .value_of("pool_idle_timeout")
                .map(|s| std::time::Duration::from_secs(s.parse::<u64>().unwrap())),
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
