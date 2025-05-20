use serde_json::Result;
use std::collections::HashMap;

#[derive(Debug)]
struct Database {
    name: String,
    collections: Vec<Collection>,
}

#[derive(Debug)]
struct Collection {
    name: String,
    documents: HashMap<i64, Document>,
}

#[derive(Debug)]
struct Document {
    id: i64,
    contents: String,
}

// Checks syntax of json &str to see if its valid.
fn is_valid_json(json_str: &str) -> String {
    match serde_json::from_str::<serde_json::Value>(json_str) {
        Ok(_) => json_str.to_string(),
        Err(_) => String::from("invalid json format"),
    }
}

// Database engine that holds a vector of databases and performs
// high level operations like creating, viewing, and deleting databases.
struct DatabaseEngine {
    databases: Vec<Database>, //TODO: Create a file directory structure to database engine for nav.
    user_dir: String,
}

impl DatabaseEngine {
    fn create_database(&mut self) {
        self.databases.push(Database { name: String::from("new database name"), collections: Vec::<Collection>::new() });
    }

    fn list_databases(&self) {
        let mut count = 0;
        for database in &self.databases {
            println!("{} {:?}", count, database); 
            
            count += 1;
        }
    }
}

fn man_page() {
    println!(
        "User Commands\n
        mkdb - create database\n
        ls - list databases\n
        man - open manual\n
        exit - exit database engine\n
        ");
}

fn run() {
    use text_io::read;
    let mut database_engine = DatabaseEngine { databases: Vec::<Database>::new(), user_dir: String::from("/") };
    loop {
        // Process user input and forward input to it's corrolated function.
        let input: String = read!();

        match input.as_str() {
            "mkdb" => database_engine.create_database(), //TODO: add additional argument for name.
            "ls" => database_engine.list_databases(),
            "man" => man_page(),
            "exit" => return,
            _ => println!("unknown command. Try man, to view the list of possible commands."),
        }
    }
}

fn main() { // TODO: Consider adding return type to main fn and error handling.
    run();
}
