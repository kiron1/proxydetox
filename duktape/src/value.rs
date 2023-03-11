use std::fmt;
use std::fmt::{Error, Formatter};

#[derive(Debug, Default)]
pub enum Value {
    #[default]
    Undefined,
    Null,
    Boolean(bool),
    Number(f64),
    String(std::string::String),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match *self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Boolean(k) => write!(f, "{k}"),
            Value::Number(k) => write!(f, "{k}"),
            Value::String(ref k) => write!(f, "{k}"),
        }
    }
}

impl From<bool> for Value {
    fn from(k: bool) -> Self {
        Value::Boolean(k)
    }
}

impl From<f64> for Value {
    fn from(k: f64) -> Self {
        Value::Number(k)
    }
}

impl From<&String> for Value {
    fn from(k: &String) -> Self {
        Value::String(k.clone())
    }
}

//impl From<str> for Value {
//    fn from(k: &str) -> Self {
//        Value::String(k.clone())
//    }
//}
