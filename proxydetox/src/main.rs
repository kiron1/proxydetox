pub mod client;
pub mod detox;

use std::boxed::Box;
use std::fs::File;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::result::Result;

use hyper::Server;
use structopt::StructOpt;

use crate::detox::DetoxService;

#[derive(Debug, StructOpt)]
#[structopt(name = "proxydetox", about = "Proxy tamer")]
struct Opt {
    #[structopt(parse(from_os_str))]
    pac_file: PathBuf,
    #[structopt(default_value = "3128")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let opt = Opt::from_args();

    let pac_script = {
        let mut pac_file = File::open(&opt.pac_file)?;
        let mut contents = String::new();
        pac_file.read_to_string(&mut contents)?;
        contents
    };

    let addr = SocketAddr::from(([127, 0, 0, 1], opt.port));
    let server = Server::bind(&addr).serve(DetoxService::new(&pac_script));
    log::info!("Listening on http://{}", addr);

    if let Err(e) = server.await {
        log::error!("server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
