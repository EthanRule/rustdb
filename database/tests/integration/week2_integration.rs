use database::{Document, Value, bson::{BsonEncoder, BsonDecoder}};
use std::collections::BTreeMap;
use std::io::Cursor;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_document_lifecycle() {
    // 1. Document Creation and Modification
    let mut doc = Document::new();
    
    // Add primitive values
    doc.set("null", Value::Null);
    doc.set("bool", Value::Bool(true));
    doc.set("int32", Value::I32(42));
    doc.set("int64", Value::I64(i64::MAX));
    doc.set("double", Value::F64(3.14159));
    doc.set("string", Value::String("Hello, BSON!".to_string()));
    
    // Add array with mixed types
    let array = vec![
        Value::I32(1),
        Value::String("array_string".to_string()),
        Value::Bool(false),
        Value::Null,
    ];
    doc.set("array", Value::Array(array));
    
    // Add nested document
    let mut nested_map = BTreeMap::new();
    nested_map.insert("nested_field".to_string(), Value::I32(99));
    nested_map.insert("nested_string".to_string(), Value::String("nested value".to_string()));
    doc.set("nested_doc", Value::Object(nested_map));
    
    // 2. Document Access and Validation
    assert_eq!(doc.get("int32"), Some(&Value::I32(42)));
    assert_eq!(doc.get("bool"), Some(&Value::Bool(true)));
    assert_eq!(doc.get("nonexistent"), None);
    
    // Test nested access
    if let Some(Value::Object(nested)) = doc.get("nested_doc") {
        assert_eq!(nested.get("nested_field"), Some(&Value::I32(99)));
    } else {
        panic!("Expected nested document");
    }
    
    // Test array access
    if let Some(Value::Array(arr)) = doc.get("array") {
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0], Value::I32(1));
    } else {
        panic!("Expected array");
    }
    
    // 3. Serialization with Progress Tracking
    let mut buffer = Cursor::new(Vec::new());
    let bytes_written = AtomicUsize::new(0);
    let mut encoder = BsonEncoder::new(&mut buffer);
    
    encoder = encoder.with_progress_callback(move |written, total| {
        bytes_written.store(written, Ordering::SeqCst);
        if total != 0 && written == total {
            assert_eq!(written, total, "Final bytes written should match total");
        }
    });
    
    encoder.encode_document(&doc).expect("Failed to encode document");
    let bytes = buffer.into_inner();
    assert!(!bytes.is_empty(), "Serialized document should not be empty");
    
    // 4. Deserialization with Progress Tracking
    let bytes_read = AtomicUsize::new(0);
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    
    decoder = decoder.with_progress_callback(move |read, total| {
        bytes_read.store(read, Ordering::SeqCst);
        assert!(read <= total, "Read bytes should not exceed total");
    });
    
    let deserialized = decoder.decode_document().expect("Failed to decode document");
    
    // 5. Verify Roundtrip Equality
    assert_eq!(doc.get("null"), deserialized.get("null"));
    assert_eq!(doc.get("bool"), deserialized.get("bool"));
    assert_eq!(doc.get("int32"), deserialized.get("int32"));
    assert_eq!(doc.get("int64"), deserialized.get("int64"));
    assert_eq!(doc.get("double"), deserialized.get("double"));
    assert_eq!(doc.get("string"), deserialized.get("string"));
    assert_eq!(doc.get("array"), deserialized.get("array"));
    assert_eq!(doc.get("nested_doc"), deserialized.get("nested_doc"));
    
    // 6. Partial Document Operations
    let fields_of_interest = vec!["string", "int32", "nested_doc"];
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    let partial = decoder.decode_partial_document(&fields_of_interest)
        .expect("Failed to decode partial document");
    
    // Verify partial document
    assert_eq!(partial.get("string"), doc.get("string"));
    assert_eq!(partial.get("int32"), doc.get("int32"));
    assert_eq!(partial.get("nested_doc"), doc.get("nested_doc"));
    assert_eq!(partial.get("bool"), None); // Should not be included
    
    // 7. Field Name Extraction
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    let field_names = decoder.get_field_names().expect("Failed to get field names");
    assert!(field_names.contains(&"string".to_string()));
    assert!(field_names.contains(&"nested_doc".to_string()));
    assert!(field_names.contains(&"array".to_string()));
    
    // 8. Document Modification and Re-serialization
    let mut modified = deserialized;
    modified.set("new_field", Value::String("new value".to_string()));
    modified.set("int32", Value::I32(100)); // Modify existing field
    
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = BsonEncoder::new(&mut buffer);
    encoder.encode_document(&modified).expect("Failed to encode modified document");
    
    // 9. Multiple Document Streaming
    let mut multiple_docs = Vec::new();
    multiple_docs.extend_from_slice(&bytes);
    multiple_docs.extend_from_slice(&bytes);
    
    let mut decoder = BsonDecoder::new(Cursor::new(&multiple_docs));
    let docs: Vec<_> = decoder.decode_documents().collect::<Result<_, _>>().expect("Failed to decode multiple documents");
    
    assert_eq!(docs.len(), 2, "Should decode two documents");
    assert_eq!(docs[0].get("int32"), docs[1].get("int32")); // Documents should be identical
    
    // 10. Error Handling
    // Test invalid UTF-8
    let mut bad_bytes = bytes.clone();
    if let Some(pos) = bad_bytes.iter().position(|&b| b == b'H') {
        bad_bytes[pos] = 0xFF;
    }
    let decode_result = BsonDecoder::new(Cursor::new(&bad_bytes)).decode_document();
    assert!(decode_result.is_err(), "Should fail to decode invalid UTF-8");
    
    // Test document too large
    let huge_string = "x".repeat(17 * 1024 * 1024); // > 16MB
    let mut huge_doc = Document::new();
    huge_doc.set("huge", Value::String(huge_string));
    
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = BsonEncoder::new(&mut buffer);
    let encode_result = encoder.encode_document(&huge_doc);
    assert!(encode_result.is_err(), "Should fail to encode too large document");
}

#[test]
fn test_document_error_handling() {
    // 1. Test invalid document length
    let invalid_length = vec![0xFF, 0xFF, 0xFF, 0xFF]; // Invalid length
    let result = BsonDecoder::new(Cursor::new(&invalid_length)).decode_document();
    assert!(result.is_err());
    
    // 2. Test missing null terminator
    let mut doc = Document::new();
    doc.set("key", Value::String("value".to_string()));
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = BsonEncoder::new(&mut buffer);
    encoder.encode_document(&doc).unwrap();
    let mut bytes = buffer.into_inner();
    
    // Remove null terminator
    if let Some(pos) = bytes.iter().position(|&b| b == 0) {
        bytes.remove(pos);
    }
    let result = BsonDecoder::new(Cursor::new(&bytes)).decode_document();
    assert!(result.is_err());
    
    // 3. Test nested document too deep
    let mut deep_doc = Document::new();
    let mut current_map = BTreeMap::new();
    
    // Create a deeply nested structure
    for i in 0..1000 {
        let mut next_map = BTreeMap::new();
        next_map.insert(format!("level_{}", i), Value::Object(current_map));
        current_map = next_map;
    }
    deep_doc.set("root", Value::Object(current_map));
    
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = BsonEncoder::new(&mut buffer);
    let result = encoder.encode_document(&deep_doc);
    assert!(result.is_err(), "Should fail due to excessive nesting");
    
    // 4. Test partial document with missing fields
    let mut doc = Document::new();
    doc.set("field1", Value::I32(1));
    doc.set("field2", Value::I32(2));
    
    let mut buffer = Cursor::new(Vec::new());
    let mut encoder = BsonEncoder::new(&mut buffer);
    encoder.encode_document(&doc).unwrap();
    let bytes = buffer.into_inner();
    
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    let result = decoder.decode_partial_document(&["field1", "nonexistent"]);
    assert!(result.is_err(), "Should fail due to missing field");
}

#[test]
fn test_document_streaming() {
    // Create a large document for streaming tests
    let mut large_doc = Document::new();
    for i in 0..1000 {
        large_doc.set(format!("field_{}", i), Value::String("x".repeat(100)));
    }
    
    // Test streaming serialization with progress tracking
    let mut buffer = Cursor::new(Vec::new());
    let progress_points = Arc::new(AtomicUsize::new(0));
    let progress_points_clone = Arc::clone(&progress_points);
    let mut encoder = BsonEncoder::new(&mut buffer);
    
    encoder = encoder.with_progress_callback(move |written, total| {
        progress_points_clone.fetch_add(1, Ordering::SeqCst);
        assert!(written <= total);
    });
    
    let result = encoder.encode_document(&large_doc);
    assert!(result.is_ok());
    let bytes = buffer.into_inner();
    assert!(progress_points.load(Ordering::SeqCst) > 0);
    
    // Test streaming deserialization
    let read_progress = Arc::new(AtomicUsize::new(0));
    let read_progress_clone = Arc::clone(&read_progress);
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    
    decoder = decoder.with_progress_callback(move |read, total| {
        read_progress_clone.fetch_add(1, Ordering::SeqCst);
        assert!(read <= total);
    });
    
    let result = decoder.decode_document();
    assert!(result.is_ok());
    assert!(read_progress.load(Ordering::SeqCst) > 0);
    
    // Test partial document streaming
    let fields: Vec<String> = (0..10).map(|i| format!("field_{}", i)).collect();
    let fields_ref: Vec<&str> = fields.iter().map(|s| s.as_str()).collect();
    
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    let partial = decoder.decode_partial_document(&fields_ref);
    assert!(partial.is_ok());
    let partial_doc = partial.unwrap();
    assert_eq!(partial_doc.get(&fields[0]), large_doc.get(&fields[0]));
    
    // Test field name extraction
    let mut decoder = BsonDecoder::new(Cursor::new(&bytes));
    let field_names = decoder.get_field_names().unwrap();
    assert!(field_names.contains(&"field_0".to_string()));
    assert!(field_names.contains(&format!("field_{}", 999)));
    assert_eq!(field_names.len(), 1000);
} 