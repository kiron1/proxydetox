use clap::Parser;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;
use toml::Value;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name to print for value
    #[arg(short, long)]
    name: Option<String>,

    /// Path of TOML file
    #[arg(short, long)]
    file: PathBuf,

    /// Path of TOML object to retrive
    path: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let toml_str = read_to_string(&args.file).expect("reading Toml file");

    let mut value = toml_str.parse::<Value>().expect("valid TOML content");

    for p in args.path {
        value = match value {
            Value::Table(t) => t.get(&p).expect("Table got entry").clone(),
            Value::Array(a) => a
                .get(p.parse::<usize>().expect("Index into array"))
                .expect("Valid index into array")
                .clone(),
            _ => panic!("Expect a table or array to perform index"),
        };
    }
    let value = match value {
        Value::String(v) => format!("{v}"),
        Value::Integer(v) => format!("{v}"),
        Value::Float(v) => format!("{v}"),
        Value::Boolean(v) => format!("{v}"),
        x => format!("{x}"),
    };
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    if let Some(ref name) = args.name {
        handle.write_all(name.as_bytes())?;
        handle.write_all(b"=")?;
    }
    handle.write_all(value.as_bytes())?;
    handle.write_all(b"\n")?;

    Ok(())
}
