use clap::Parser;
use std::fs::read_to_string;
use std::io::Write;
use std::path::PathBuf;
use toml::{Table, Value};

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
    #[arg(num_args(1..))]
    path: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let toml_str = read_to_string(&args.file).expect("reading TOML file");

    let document = toml_str.parse::<Table>().expect("valid TOML content");
    let mut value = Value::from(document);

    for p in args.path.iter() {
        value = match value {
            Value::Table(t) => {
                let keys = t
                    .keys()
                    .map(String::from)
                    .reduce(|acc, s| format!("{acc}, {s}"))
                    .unwrap_or_default();
                t.get(p)
                    .ok_or_else(|| format!("Table has no entry {p}, but have {keys}"))
                    .expect("Table access")
                    .clone()
            }
            Value::Array(a) => a
                .get(p.parse::<usize>().expect("Index into array"))
                .expect("Valid index into array")
                .clone(),
            _ => panic!("Expect a table or array to perform index"),
        };
    }
    let value = match value {
        Value::String(v) => v.to_string(),
        Value::Integer(v) => v.to_string(),
        Value::Float(v) => v.to_string(),
        Value::Boolean(v) => v.to_string(),
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
