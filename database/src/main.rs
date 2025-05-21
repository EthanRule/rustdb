use serde_json::Result;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};

#[derive(Debug)]
struct Database {
    name: String,
    collections: HashMap<String, Collection>,
}

impl Database {
    fn list_collections(&self) {
        println!("{:?}", self.collections);
    }
}

#[derive(Debug)]
struct Collection {
    name: String,
    documents: Vec<Document>,
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
    databases: HashMap<String, Database>,
    database_path: String,
}

impl DatabaseEngine {
    fn create_database(&mut self, input: &str) {

        let name = &input.to_string()[4..input.len()].trim().to_string();

        if name.contains(' ') {
            println!("Error: database name contains spaces");
            return;
        }

        let new_db = Database {
            name: name.to_string(),
            collections: HashMap::<String, Collection>::new()
        };
        
        if self.databases.contains_key(&name.to_string()) {
            println!("Error: database with name {} already exists", &new_db.name);
            return;
        } else {
            self.databases.insert(name.to_string(), new_db);
        }
    }

    fn create_collection(&self) {
        
    }

    fn list_databases(&self) {
        // TODO: be aware of if we are at root or if we are inside a database on what to display
        // with ls

        let mut count = 0;
        for database in &self.databases {
            println!("{} {:?}", count, database); 
            
            count += 1;
        }
    }

    fn change_directory(&mut self, input: &str) {
        let path = &input.to_string()[3..input.len()].trim().to_string();
        // check if we are at root => enter a database
        // check if we are at database => enter collection

        if self.database_path == "/" {
            // check if the database exists, if so update path, if not print non existant msg
            if self.databases.contains_key(path) {
                self.database_path = self.database_path.to_owned() + &path + "/";
            } else {
                println!("Error: database {} does not exist.", path);
            }
        }
        else { // are already inside a database
            let current_db = &self.database_path[2..self.database_path.len()];

            // check if the collection exists, if so change the path to it.
            if self.databases[current_db].collections.contains_key(path) {
                self.database_path = self.database_path.to_owned() + &path + "/"
            } else {
                println!("Error: collection {} does not exist.", path);
            }
        }

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
    let mut database_engine = DatabaseEngine { databases: HashMap::<String, Database>::new(), database_path: String::from("/") };
    loop {
        print!("{}>", database_engine.database_path);
        io::stdout().flush().expect("failed to flush output");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read into input buffer.");
        let input = input.trim();

        match input {
            x if x.starts_with("mkdb") => database_engine.create_database(x),
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
