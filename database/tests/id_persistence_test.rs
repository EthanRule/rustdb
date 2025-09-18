use database::document::Document;
use database::document::bson::{serialize_document, deserialize_document};
use database::document::types::Value;

#[test]
fn test_document_id_persistence() {
    // Create a document
    let mut doc = Document::new();
    doc.set("name", Value::String("Alice".to_string()));
    doc.set("age", Value::I32(25));
    
    // Get the original ID
    let original_id = doc.id().clone();
    
    // Serialize and deserialize
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    // ID should be preserved
    assert_eq!(deserialized.id(), &original_id);
    
    // Data should also be preserved
    assert_eq!(deserialized.get("name"), Some(&Value::String("Alice".to_string())));
    assert_eq!(deserialized.get("age"), Some(&Value::I32(25)));
}

#[test]
fn test_multiple_roundtrips_preserve_id() {
    let mut doc = Document::new();
    doc.set("test", Value::String("value".to_string()));
    
    let original_id = doc.id().clone();
    
    // Multiple roundtrips
    for _ in 0..5 {
        let serialized = serialize_document(&doc).unwrap();
        doc = deserialize_document(&serialized).unwrap();
        
        // ID should remain the same
        assert_eq!(doc.id(), &original_id);
    }
}

#[test]
fn test_empty_document_id_persistence() {
    let doc = Document::new();
    let original_id = doc.id().clone();
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    // Even empty documents should preserve their ID
    assert_eq!(deserialized.id(), &original_id);
    assert_eq!(deserialized.len(), 0);
}