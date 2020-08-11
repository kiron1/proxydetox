pub mod auth;
pub mod client;
pub mod detox;
pub mod io;
pub mod net;

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
    env_logger::init();

    let opt: Opt = argh::from_env();

    let (pac_path, pac_script) = load_pac_file(&opt);
    if let Some(path) = pac_path {
        log::info!("PAC path: {}", path.canonicalize()?.display());
    } else {
        log::info!(
            "Using inline PAC config: {}",
            pac_script.as_ref().expect("inline PAC config")
        );
    }

    let addr = SocketAddr::from(([127, 0, 0, 1], opt.port));
    let server = Server::bind(&addr).serve(DetoxService::new(&pac_script?));
    log::info!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        log::error!("server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
