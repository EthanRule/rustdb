# Rust Database Engine

A high-performance database engine written in Rust, focusing on efficient BSON document storage and retrieval.

## Week 2 Progress: BSON Serialization Implementation

### Completed Features

1. **Zero-Copy Deserialization**
   - Implemented streaming deserialization to minimize memory allocations
   - Added support for partial document reading
   - Optimized field name extraction

2. **Memory Optimizations**
   - Added document size validation
   - Implemented memory limits for document operations
   - Added nesting depth limits to prevent stack overflow
   - Optimized string allocation patterns

3. **Performance Features**
   - Streaming array/object encoding
   - Efficient handling of nested documents
   - Buffer reuse for string and binary data
   - Progress tracking callbacks for long operations

### API Improvements

1. **Document Operations**
   ```rust
   // Create and modify documents
   let mut doc = Document::new();
   doc.set("field", Value::String("value".to_string()));
   
   // Access document fields
   if let Some(value) = doc.get("field") {
       println!("Found value: {}", value);
   }
   ```

2. **Serialization**
   ```rust
   // Serialize with progress tracking
   let mut encoder = BsonEncoder::new(buffer);
   encoder.with_progress_callback(|written, total| {
       println!("Progress: {}/{}", written, total);
   });
   encoder.encode_document(&doc)?;
   ```

3. **Partial Document Reading**
   ```rust
   // Read only specific fields
   let fields = vec!["field1", "field2"];
   let partial = decoder.decode_partial_document(&fields)?;
   ```

### Performance Characteristics

1. **Memory Usage**
   - Document size validation (max 16MB)
   - Streaming operations for large documents
   - Efficient string handling

2. **CPU Efficiency**
   - Zero-copy operations where possible
   - Minimal data copying during serialization
   - Efficient nested document handling
   - Optimized string processing

3. **Throughput**
   - Streaming support for large documents
   - Progress tracking for long operations
   - Partial document reading support

### Error Handling

1. **Validation**
   - Document size limits
   - Field name validation
   - UTF-8 string validation
   - Nesting depth limits

2. **Error Types**
   - IO errors
   - Memory limit errors
   - Invalid data errors
   - Missing field errors

### Testing

1. **Unit Tests**
   - Document lifecycle tests
   - Error handling tests
   - Streaming operation tests
   - Memory limit tests

2. **Integration Tests**
   - End-to-end document operations
   - Serialization/deserialization
   - Error handling scenarios
   - Performance characteristics

## Getting Started

1. **Installation**
   ```bash
   git clone <repository-url>
   cd rust_database_engine
   cargo build --release
   ```

2. **Running Tests**
   ```bash
   cargo test
   cargo bench  # Run benchmarks
   ```

3. **Example Usage**
   ```rust
   use database::{Document, Value};
   
   // Create a document
   let mut doc = Document::new();
   doc.set("name", Value::String("example".to_string()));
   
   // Serialize
   let mut encoder = BsonEncoder::new(buffer);
   encoder.encode_document(&doc)?;
   
   // Deserialize
   let mut decoder = BsonDecoder::new(buffer);
   let doc = decoder.decode_document()?;
   ```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

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
- [x] Testing and Benchmarks for Types & Documents
- [x] BSON Serialization/Deserialization

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

### Benchmark Results

1. **Document Serialization**
   - Small documents (10 fields): ~4 µs
   - Medium documents (100 fields): ~35 µs
   - Large documents (1000 fields): ~370 µs
   - Very large documents (10000 fields): ~6 ms
   - Nested documents (depth 5-50): 0.6-3.5 µs
   - Mixed type documents: ~1 µs

2. **Document Deserialization**
   - Small documents (10 fields): ~10 µs
   - Medium documents (100 fields): ~130 µs
   - Large documents (1000 fields): ~1.7 ms
   - Very large documents (10000 fields): ~21 ms
   - Nested documents (depth 5-50): 1.8-16 µs
   - Mixed type documents: ~2.3 µs

3. **Partial Document Operations**
   - Small documents (100 fields):
     * 1 field: ~30 µs
     * 10 fields: ~37 µs
     * 50 fields: ~80 µs
   - Large documents (10000 fields):
     * 1 field: ~4.4 ms
     * 10 fields: ~4.9 ms
     * 50 fields: ~7.3 ms

4. **Streaming Operations**
   - Document size scaling:
     * 1000 fields: ~400 µs encode, ~1.8 ms decode
     * 10000 fields: ~6 ms encode, ~22 ms decode
     * 100000 fields: ~76 ms encode, ~240 ms decode
   - Multiple document streaming: ~0.5 µs per document

5. **Field Name Extraction**
   - 100 fields: ~30 µs
   - 1000 fields: ~500 µs


