# Rust Database Engine

## Description
NoSQL lightweight database easy to use, with a focus on performance and simplicity. The engine supports basic  
CRUD operations, indexing, querying, and transactions. It is built to resemeble MongoDB's system of collections  
and documents.

## Inspirations
- MongoDB
- PostgreSQL

## Current Progress
- [x] Database Types
- [x] Document Struct that holds database types
- [x] Testing and Bench  marks for Types & Docuements.

## Examples
#[allow(dead_code)]
```rust

fn example_organization_structure() -> Document {
    let mut user1 = BTreeMap::new();
    user1.insert("name".to_string(), Value::String("Charlie".to_string()));
    user1.insert("role".to_string(), Value::String("Developer".to_string()));

    let mut user2 = BTreeMap::new();
    user2.insert("name".to_string(), Value::String("Dana".to_string()));
    user2.insert("role".to_string(), Value::String("Designer".to_string()));

    let team_members = vec![Value::Object(user1), Value::Object(user2)];

    let mut team = BTreeMap::new();
    team.insert("name".to_string(), Value::String("Frontend".to_string()));
    team.insert("members".to_string(), Value::Array(team_members));

    let mut org = BTreeMap::new();
    org.insert(
        "org_name".to_string(),
        Value::String("Acme Corp".to_string()),
    );
    org.insert("teams".to_string(), Value::Array(vec![Value::Object(team)]));

    Document {
        data: org,
        id: Value::ObjectId(ObjectId::new()),
    }
}
```


