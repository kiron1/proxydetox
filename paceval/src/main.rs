use http::Uri;
use paclib::Evaluator;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "paceval", about = "Evaluate a PAC JavaSciript file.")]
struct Opt {
    #[structopt(parse(from_os_str))]
    pac_file: PathBuf,
    urls: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opt::from_args();

    let pac_content = {
        let mut pac_file = File::open(&opt.pac_file)?;
        let mut contents = String::new();
        pac_file.read_to_string(&mut contents)?;
        contents
    };

    let mut pac = Evaluator::new(&pac_content)?;

    for url in opt.urls {
        let uri = url.parse::<Uri>()?;
        let proxies = pac.find_proxy(&uri)?;
        println!("{}: {}", uri, proxies);
    }
    Ok(())
}
