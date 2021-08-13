use std::{
    ffi::OsString,
    fs::read_to_string,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use clap::{App, AppSettings, Arg, ArgMatches};

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

fn is_num<T: FromStr + PartialOrd>(v: String) -> Result<(), String> {
    match v.parse::<T>() {
        Ok(_v) => Ok(()),
        Err(ref _cause) => Err(format!("invalid number")),
    }
}

fn is_file(v: String) -> Result<(), String> {
    if Path::new(&v).is_file() {
        Ok(())
    } else {
        Err(format!("file '{}' does not exists", &v))
    }
}

impl Options {
    pub fn load() -> Self {
        let default_pool_max_idle_per_host = usize::MAX.to_string();
        let default_netrc_file = {
            let mut netrc_path = dirs::home_dir().unwrap_or_default();
            netrc_path.push(".netrc");
            netrc_path.to_str().unwrap_or_default().to_owned()
        };

        let app: _ = App::new(env!("CARGO_PKG_NAME"))
            .version(env!("CARGO_PKG_VERSION"))
            .about("A small proxy to relive the pain of some corporate proxies")
            .setting(AppSettings::AllArgsOverrideSelf);

        #[cfg(feature = "negotiate")]
        let app = app.arg(
            Arg::with_name("negotiate")
                .short("n")
                .long("negotiate")
                .help("Enables Negotiate (SPNEGO) authentication"),
        );

        let netrc_arg = Arg::with_name("netrc_file")
            .long("netrc-file")
            .help("Path to a .netrc file to be used for basic authentication")
            .validator(is_file)
            .default_value(&default_netrc_file)
            .takes_value(true);
        #[cfg(feature = "negotiate")]
        let netrc_arg = netrc_arg.conflicts_with("negotiate");

        let app = app
            .arg(
                Arg::with_name("port")
                    .short("P")
                    .long("port")
                    .help("Listening port")
                    .validator(is_num::<u16>)
                    .default_value("3128"),
            )
            .arg(
                Arg::with_name("pac_file")
                    .long("pac-file")
                    .short("p")
                    .help(
                        "PAC file to be used to decide which upstream proxy to forward the request",
                    )
                    .validator(is_file)
                    .takes_value(true),
            )
            .arg(netrc_arg)
            .arg(
                Arg::with_name("always_use_connect")
                    .short("c")
                    .long("always_use_connect")
                    .help("Always use CONNECT method even for http:// resources"),
            )
            .arg(
                Arg::with_name("pool_max_idle_per_host")
                    .long("pool-max-idle-per-host")
                    .help("Maximum idle connection per host allowed in the pool")
                    .validator(is_num::<usize>)
                    .default_value(&default_pool_max_idle_per_host),
            )
            .arg(
                Arg::with_name("pool_idle_timeout")
                    .long("pool-idle-timeout")
                    .help("Optional timeout for idle sockets being kept-alive")
                    .validator(is_num::<u64>)
                    .takes_value(true),
            );

        let mut args = readrc();
        args.extend(std::env::args_os());
        let matches = app.get_matches_from(args);

        matches.into()
    }
}

impl From<ArgMatches<'_>> for Options {
    fn from(m: ArgMatches) -> Self {
        Self {
            #[cfg(feature = "negotiate")]
            negotiate: m.is_present("negotiate"),
            pac_file: m.value_of("pac_file").map(|s| String::from(s)),
            netrc_file: m.value_of("netrc_file").map(|s| PathBuf::from(s)).unwrap(),
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
    ];
    for path in config_locations {
        if let Ok(content) = read_to_string(&path) {
            // todo: this will fail with arguments which require a space (e.g. path of pac_file)
            let args = content
                .split('\n')
                .map(|s| s.trim())
                .filter(|s| !s.starts_with('#'))
                .map(str::split_ascii_whitespace)
                .flatten()
                .filter(|s| !s.is_empty())
                .map(|s| OsString::from(s))
                .collect::<Vec<_>>();
            return args;
        }
    }
    Vec::new()
}
