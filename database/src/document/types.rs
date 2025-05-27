// This file defines the `Value` enum used in the database system.
// There are several functions to handle type comparisons and conversions.

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    String(String),
}

use std::fmt;

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::I32(i) => write!(f, "{}", i),
            Value::I64(i) => write!(f, "{}", i),
            Value::F64(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

fn is_null(value: &Value) -> bool {
    matches!(value, Value::Null)
}

fn is_bool(value: &Value) -> bool {
    matches!(value, Value::Bool(_))
}

fn is_number(value: &Value) -> bool {
    matches!(value, Value::I32(_) | Value::I64(_) | Value::F64(_))
}

fn is_string(value: &Value) -> bool {
    matches!(value, Value::String(_))
}

fn as_bool(value: &Value) -> Option<bool> {
    match value {
        Value::Null => Some(false),
        Value::I32(x) => match x {
            val if *val == 1 => Some(true),
            val if *val == 0 => Some(false),
            _ => None,
        },
        Value::I64(x) => match x {
            val if *val == 1 => Some(true),
            val if *val == 0 => Some(false),
            _ => None,
        },
        Value::F64(x) => match x {
            val if *val == 1.0 => Some(true),
            val if *val == 0.0 => Some(false),
            _ => None,
        },
        Value::String(x) => match x {
            val if val.to_lowercase() == String::from("true") => Some(true),
            val if val.to_lowercase() == String::from("false") => Some(false),
            _ => None,
        }
        Value::Bool(x) => Some(*x),
    }
}


