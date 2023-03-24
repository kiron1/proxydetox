use clap::Parser;
use http::Uri;
use paclib::Engine;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Parser)]
/// Evaluate a PAC JavaSciript file
struct Opt {
    /// path to a PAC file
    pac_file: PathBuf,

    /// list of URIs to evaluate
    urls: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::parse();

    let pac_content = {
        let mut pac_file = File::open(&opt.pac_file)?;
        let mut contents = String::new();
        pac_file.read_to_string(&mut contents)?;
        contents
    };

    let mut pac = Engine::with_pac_script(&pac_content)?;

    for url in opt.urls {
        let uri = url.parse::<Uri>()?;
        let proxies = pac.find_proxy(&uri)?;
        println!("{uri}: {proxies}");
    }
    Ok(())
}
