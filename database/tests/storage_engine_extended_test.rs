use database::{Document, Value, storage::storage_engine::{StorageEngine, DocumentId}};
use tempfile::tempdir;

#[test]
fn test_document_id_accessors() {
    // Test that DocumentId accessors work
    let doc_id = DocumentId::new(42, 123);
    assert_eq!(doc_id.page_id(), 42);
    assert_eq!(doc_id.slot_id(), 123);
}

#[test]
fn test_insert_document_with_existing_page() {
    // This test would work if we had a way to pre-populate the buffer pool with pages
    // For now, this is a placeholder to show how the API should work in the future
    
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    
    // Create database file
    let _db_file = database::storage::file::DatabaseFile::create(&db_path)
        .expect("Failed to create database file");
    drop(_db_file);
    
    let mut storage_engine = StorageEngine::new(&db_path, 10)
        .expect("Failed to create storage engine");
    
    // Create a simple document
    let mut doc = Document::new();
    doc.set("test", Value::String("data".to_string()));
    
    // Try to insert - should fail because no pages exist
    let result = storage_engine.insert_document(&doc);
    assert!(result.is_err());
    
    // The error should be about no existing pages
    let error = result.unwrap_err();
    assert!(error.to_string().contains("No existing page has sufficient space"));
}
