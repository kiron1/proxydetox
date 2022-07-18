use argh::FromArgs;
use http::Uri;
use paclib::Evaluator;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, FromArgs)]
/// Evaluate a PAC JavaSciript file
struct Opt {
    /// path to a PAC file
    #[argh(positional)]
    pac_file: PathBuf,

    /// list of URIs to evaluate
    #[argh(positional)]
    urls: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt: Opt = argh::from_env();

    let pac_content = {
        let mut pac_file = File::open(&opt.pac_file)?;
        let mut contents = String::new();
        pac_file.read_to_string(&mut contents)?;
        contents
    };

    let mut pac = Evaluator::with_pac_script(&pac_content)?;

    for url in opt.urls {
        let uri = url.parse::<Uri>()?;
        let proxies = pac.find_proxy(&uri)?;
        println!("{}: {}", uri, proxies);
    }
    Ok(())
}
