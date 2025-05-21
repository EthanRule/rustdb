use serde_json::Result;
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Debug)]
struct Database {
    name: String,
    collections: Vec<Collection>,
}

impl Database {
    fn list_collections(&self) {
        println!("{:?}", self.collections);
    }
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
    database_path: String,
}

impl DatabaseEngine {
    fn create_database(&mut self) {
        // Ensure new db names have no spaces.
        self.databases.push(Database { name: String::from("name"), collections: Vec::<Collection>::new() });
    }

    fn list_databases(&self) {
        let mut count = 0;
        for database in &self.databases {
            println!("{} {:?}", count, database); 
            
            count += 1;
        }
    }

    fn change_directory(&self, input: &str) {
        
    }
}

fn man_page() {
    println!(
        "User Commands\n
        mkdb - create database\n
        ls - list databases\n
        cd - change directory\n
        man - open manual\n
        exit - exit database engine\n
        ");
}

fn run() {
    let mut database_engine = DatabaseEngine { databases: Vec::<Database>::new(), database_path: String::from("/") };
    loop {
        print!("{}>", database_engine.database_path);
        io::stdout().flush().expect("failed to flush output");

        // Process user input and forward input to it's corrolated function.
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read into input buffer.");
        let input = input.trim();

        match input {
            "mkdb" => database_engine.create_database(), //TODO: add additional argument for
            "ls" => database_engine.list_databases(),
            x if x.starts_with("cd") => database_engine.change_directory(x),
            "man" => man_page(),
            "exit" => return,
            _ => println!("unknown command. Try man, to view the list of possible commands."),
        }
    }
}

fn main() { // TODO: Consider adding return type to main fn and error handling.
    run();
}
