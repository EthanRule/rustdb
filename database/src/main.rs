use serde_json::Result;
use std::collections::HashMap;

struct Database {
    name: String,
    collections: Vec<Collection>,
}

struct Collection {
    name: String,
    documents: HashMap<i64, Document>,
}

struct Document {
    id: i64,
    contents: String,
}

fn is_valid_json(json_str: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(_) => json_str.to_string(),
        Err(_) => String::from("invalid json format"),
    }
}

fn main() {
    let valid_json = r#"{"name": "John", "age": 30}"#;
    let invalid_json = r#"{"name": "John", "age": }"#;

    let document = Document { id: 0i64, contents: is_valid_json(valid_json) };
    let invalid_document = Document { id: 0i64, contents: is_valid_json(invalid_json) };
    
    println!("{} {}", document.id, document.contents);
}
