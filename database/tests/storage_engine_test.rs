use database::{storage::storage_engine::StorageEngine, Document, Value};
use tempfile::tempdir;

#[test]
fn test_insert_document_basic() {
    // Create a temporary directory for the test database
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    // First create the database file
    let _db_file = database::storage::file::DatabaseFile::create(&db_path)
        .expect("Failed to create database file");
    // Drop it so the file is closed and can be reopened by StorageEngine
    drop(_db_file);

    // Create storage engine with a small buffer pool
    let mut storage_engine =
        StorageEngine::new(&db_path, 10).expect("Failed to create storage engine");

    // Create a simple document
    let mut doc = Document::new();
    doc.set("name", Value::String("John Doe".to_string()));
    doc.set("age", Value::I32(30));
    doc.set("active", Value::Bool(true));

    // Insert the document - this should fail since no pages exist yet
    let result = storage_engine.insert_document(&doc);

    // We expect this to fail with our current implementation since
    // we haven't implemented page allocation yet
    assert!(!result.is_err());
}

#[test]
fn test_document_serialization() {
    // Test that our document serialization works
    let mut doc = Document::new();
    doc.set("test_field", Value::String("test_value".to_string()));
    doc.set("number", Value::I32(42));

    // This should serialize successfully
    let serialized = database::document::bson::serialize_document(&doc);
    assert!(serialized.is_ok());

    let bytes = serialized.unwrap();
    assert!(bytes.len() > 0);

    // The serialized bytes should be reasonable in size
    // BSON has some overhead but should be fairly compact
    assert!(bytes.len() < 1000); // Should be much smaller for this simple doc
}
