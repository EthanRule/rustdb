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

// Information expert database engine that holds a vector
// of databases and performs high level operations like
// creating, deleting, viewing databases.
struct DatabaseEngine {
    databases: Vec<Database>,
}

impl DatabaseEngine {
    fn create(&mut self) {
        println!("creating database");
    }

    fn view(&self) {
        println!("viewing databases");
    }

    fn destroy(&mut self) {
        println!("destorying database");
    } 
}

fn run() {
    use text_io::read;
    let mut database_engine = DatabaseEngine { databases: Vec::<Database>::new() };
    loop {
        // Process user input and forward input to it's corrolated function.
        let input: String = read!();
        println!("input: {:?}", input);
        match input.as_str() {
            "create" => database_engine.create(),
            "view" => database_engine.view(),
            "destroy" => database_engine.destroy(),
            "exit" => return,
            _ => println!("unknown command. Try create, view, destroy or exit instead"),
        }
    }
}

fn main() {
    //let valid_json = r#"{"name": "John", "age": 30}"#;
    //let invalid_json = r#"{"name": "John", "age": }"#;

    //let document = Document { id: 0i64, contents: is_valid_json(valid_json) };
    //let invalid_document = Document { id: 0i64, contents: is_valid_json(invalid_json) };
    
    run();
}
