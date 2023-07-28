use std::env;
use std::fs::read_to_string;
use toml::Value;

fn main() {
    let toml_path = env::args()
        .nth(1)
        .expect("Toml file path as first argument");
    let value_path = env::args().skip(2).collect::<Vec<String>>();

    let toml_str = read_to_string(toml_path).expect("reading Toml file");

    let mut value = toml_str
        .parse::<Value>()
        .expect("valid TOML content");

    for p in value_path {
        value = match value {
            Value::Table(t) => t.get(&p).expect("Table got entry").clone(),
            Value::Array(a) => a
                .get(p.parse::<usize>().expect("Index into array"))
                .expect("Valid index into array")
                .clone(),
            _ => panic!("Expect a table or array to perform index"),
        };
    }
    match value {
        Value::String(v) => println!("{v}"),
        Value::Integer(v) => println!("{v}"),
        Value::Float(v) => println!("{v}"),
        Value::Boolean(v) => println!("{v}"),
        x => println!("{x}"),
    }
}
