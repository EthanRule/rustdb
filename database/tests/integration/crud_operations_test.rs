use database::{
    storage::storage_engine::StorageEngine,
    Document, Value,
};
use tempfile::tempdir;

#[test]
fn test_complete_crud_operations() {
    // Create a temporary directory for the test database
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_crud.db");

    // First create the database file
    let _db_file = database::storage::file::DatabaseFile::create(&db_path)
        .expect("Failed to create database file");
    drop(_db_file); // Drop it so the file is closed and can be reopened by StorageEngine

    // Create storage engine with a small buffer pool
    let mut storage_engine = StorageEngine::new(&db_path, 10)
        .expect("Failed to create storage engine");

    // Test 1: Insert documents
    println!("=== Testing INSERT operations ===");
    
    let mut doc1 = Document::new();
    doc1.set("name", Value::String("Alice".to_string()));
    doc1.set("age", Value::I32(25));
    doc1.set("active", Value::Bool(true));
    
    let mut doc2 = Document::new();
    doc2.set("name", Value::String("Bob".to_string()));
    doc2.set("age", Value::I32(30));
    doc2.set("active", Value::Bool(false));

    let doc1_id = storage_engine.insert_document(&doc1)
        .expect("Failed to insert doc1");
    let doc2_id = storage_engine.insert_document(&doc2)
        .expect("Failed to insert doc2");
    
    println!("Inserted doc1 at {:?}", doc1_id);
    println!("Inserted doc2 at {:?}", doc2_id);

    // Test 2: Read documents to verify inserts
    println!("\n=== Testing SELECT operations ===");
    let retrieved_doc1 = storage_engine.get_document(&doc1_id)
        .expect("Failed to retrieve doc1");
    let retrieved_doc2 = storage_engine.get_document(&doc2_id)
        .expect("Failed to retrieve doc2");
    
    assert_eq!(retrieved_doc1.get("name"), Some(&Value::String("Alice".to_string())));
    assert_eq!(retrieved_doc1.get("age"), Some(&Value::I32(25)));
    assert_eq!(retrieved_doc2.get("name"), Some(&Value::String("Bob".to_string())));
    assert_eq!(retrieved_doc2.get("age"), Some(&Value::I32(30)));
    
    println!("Successfully retrieved both documents");

    // Test 3: Update operations
    println!("\n=== Testing UPDATE operations ===");
    
    // Test 3a: In-place update (smaller or same size)
    let mut updated_doc1 = Document::new();
    updated_doc1.set("name", Value::String("Alice".to_string()));
    updated_doc1.set("age", Value::I32(26)); // Just change age
    updated_doc1.set("active", Value::Bool(true));
    
    let update_result1 = storage_engine.update_document(&doc1_id, &updated_doc1)
        .expect("Failed to update doc1");
    println!("Updated doc1 (in-place): {:?}", update_result1);
    
    // Verify the update
    let retrieved_updated_doc1 = storage_engine.get_document(&doc1_id)
        .expect("Failed to retrieve updated doc1");
    assert_eq!(retrieved_updated_doc1.get("age"), Some(&Value::I32(26)));
    println!("In-place update verified successfully");

    // Test 3b: Update that requires relocation (larger size)
    let mut large_updated_doc2 = Document::new();
    large_updated_doc2.set("name", Value::String("Robert".to_string())); // Longer name
    large_updated_doc2.set("age", Value::I32(31));
    large_updated_doc2.set("active", Value::Bool(true));
    large_updated_doc2.set("description", Value::String("This is a much longer description that will make the document larger and require relocation within the page or to a new page".to_string()));
    
    let update_result2 = storage_engine.update_document(&doc2_id, &large_updated_doc2)
        .expect("Failed to update doc2");
    println!("Updated doc2 (with relocation): {:?}", update_result2);
    
    // Verify the update
    let retrieved_updated_doc2 = storage_engine.get_document(&doc2_id)
        .expect("Failed to retrieve updated doc2");
    assert_eq!(retrieved_updated_doc2.get("name"), Some(&Value::String("Robert".to_string())));
    assert_eq!(retrieved_updated_doc2.get("age"), Some(&Value::I32(31)));
    assert!(retrieved_updated_doc2.get("description").is_some());
    println!("Relocation update verified successfully");

    // Test 4: Delete operations
    println!("\n=== Testing DELETE operations ===");
    
    // Delete doc1
    storage_engine.delete_document(&doc1_id)
        .expect("Failed to delete doc1");
    println!("Deleted doc1");
    
    // Verify deletion - should return error
    let delete_result = storage_engine.get_document(&doc1_id);
    assert!(delete_result.is_err());
    println!("Verified doc1 is deleted (cannot retrieve)");
    
    // Verify doc2 is still accessible
    let still_available_doc2 = storage_engine.get_document(&doc2_id)
        .expect("doc2 should still be accessible");
    assert_eq!(still_available_doc2.get("name"), Some(&Value::String("Robert".to_string())));
    println!("Verified doc2 is still accessible after doc1 deletion");

    // Test 5: Try to delete already deleted document
    let double_delete_result = storage_engine.delete_document(&doc1_id);
    assert!(double_delete_result.is_err());
    println!("Verified cannot delete already deleted document");

    // Test 6: Try to update deleted document
    let update_deleted_result = storage_engine.update_document(&doc1_id, &updated_doc1);
    assert!(update_deleted_result.is_err());
    println!("Verified cannot update deleted document");

    println!("\n=== All CRUD operations completed successfully! ===");
}

#[test]
fn test_update_edge_cases() {
    // Create a temporary directory for the test database
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test_update_edge.db");

    // First create the database file
    let _db_file = database::storage::file::DatabaseFile::create(&db_path)
        .expect("Failed to create database file");
    drop(_db_file);

    // Create storage engine
    let mut storage_engine = StorageEngine::new(&db_path, 10)
        .expect("Failed to create storage engine");

    // Insert a document
    let mut doc = Document::new();
    doc.set("id", Value::I32(1));
    doc.set("data", Value::String("original".to_string()));
    
    let doc_id = storage_engine.insert_document(&doc)
        .expect("Failed to insert document");

    // Test 1: Update to exactly same size
    let mut same_size_doc = Document::new();
    same_size_doc.set("id", Value::I32(1));
    same_size_doc.set("data", Value::String("modified".to_string())); // Same length as "original"
    
    let result = storage_engine.update_document(&doc_id, &same_size_doc)
        .expect("Failed to update with same size");
    println!("Same size update result: {:?}", result);

    // Test 2: Update to smaller size
    let mut smaller_doc = Document::new();
    smaller_doc.set("id", Value::I32(1));
    smaller_doc.set("data", Value::String("small".to_string()));
    
    let result = storage_engine.update_document(&doc_id, &smaller_doc)
        .expect("Failed to update with smaller size");
    println!("Smaller size update result: {:?}", result);

    // Verify final state
    let final_doc = storage_engine.get_document(&doc_id)
        .expect("Failed to retrieve final document");
    assert_eq!(final_doc.get("data"), Some(&Value::String("small".to_string())));

    println!("Update edge cases test completed successfully!");
}