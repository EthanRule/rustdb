use database::{Document, Value, bson::{BsonEncoder, BsonDecoder}};
use std::collections::BTreeMap;
use std::io::{Cursor, Seek};
use std::sync::atomic::{AtomicUsize, Ordering};

fn main() {
    // Create a document with various types
    let mut doc = Document::new();
    doc.set("null", Value::Null);
    doc.set("bool", Value::Bool(true));
    doc.set("int32", Value::I32(42));
    doc.set("int64", Value::I64(i64::MAX));
    doc.set("double", Value::F64(3.14159));
    doc.set("string", Value::String("Hello, BSON!".to_string()));
    
    // Create nested array
    let array = vec![
        Value::I32(1),
        Value::String("array_string".to_string()),
        Value::Bool(false),
    ];
    doc.set("array", Value::Array(array));
    
    // Create nested document
    let mut nested_map = BTreeMap::new();
    nested_map.insert("nested_field".to_string(), Value::I32(99));
    doc.set("nested_doc", Value::Object(nested_map));
    
    println!("Original document:");
    println!("{:#?}", doc);
    
    // Demonstrate streaming serialization with progress tracking
    let mut buffer = Cursor::new(Vec::new());
    let bytes_written = AtomicUsize::new(0);
    let mut encoder = BsonEncoder::new(&mut buffer);
    
    encoder = encoder.with_progress_callback(move |written, total| {
        bytes_written.store(written, Ordering::SeqCst);
        println!("Writing progress: {}/{} bytes", written, total);
    });
    
    // Serialize
    encoder.encode_document(&doc).expect("Failed to encode document");
    let bytes = buffer.into_inner();
    println!("\nSerialized size: {} bytes", bytes.len());
    
    // Demonstrate streaming deserialization
    let bytes_read = AtomicUsize::new(0);
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    
    decoder = decoder.with_progress_callback(move |read, total| {
        bytes_read.store(read, Ordering::SeqCst);
        println!("Reading progress: {}/{} bytes", read, total);
    });
    
    // Deserialize
    let deserialized = decoder.decode_document().expect("Failed to decode document");
    println!("\nDeserialized document:");
    println!("{:#?}", deserialized);
    
    // Demonstrate partial document reading
    let fields_of_interest = vec!["string", "int32", "nested_doc"];
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    let partial = decoder.decode_partial_document(&fields_of_interest)
        .expect("Failed to decode partial document");
    
    println!("\nPartial document (selected fields only):");
    println!("{:#?}", partial);
    
    // Demonstrate field name extraction
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    let field_names = decoder.get_field_names().expect("Failed to get field names");
    println!("\nDocument field names:");
    println!("{:#?}", field_names);
    
    // Demonstrate streaming multiple documents
    let mut multiple_docs = Vec::new();
    multiple_docs.extend_from_slice(&bytes);
    multiple_docs.extend_from_slice(&bytes);
    
    println!("\nReading multiple documents:");
    let mut decoder = BsonDecoder::new(Cursor::new(&multiple_docs));
    for (i, doc_result) in decoder.decode_documents().enumerate() {
        match doc_result {
            Ok(doc) => {
                let mut field_count = 0;
                for field in &field_names {
                    if doc.get(field).is_some() {
                        field_count += 1;
                    }
                }
                println!("Document {}: {} fields", i + 1, field_count);
            }
            Err(e) => println!("Error reading document {}: {}", i + 1, e),
        }
    }
} 