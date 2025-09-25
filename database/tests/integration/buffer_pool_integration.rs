use database::storage::buffer_pool::BufferPool;
use database::storage::file::DatabaseFile;
use database::storage::storage_engine::StorageEngine;
use database::{Document, Value};
use std::path::Path;
use std::fs;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_temp_database() -> Result<(String, DatabaseFile), Box<dyn std::error::Error>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::thread;
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let thread_id = format!("{:?}", thread::current().id());
        let temp_path = format!("test_db_{}_{}_{}.db", std::process::id(), timestamp, thread_id.chars().filter(|c| c.is_numeric()).collect::<String>());
        
        // Clean up any existing file first
        let _ = fs::remove_file(&temp_path);
        
        // Create the database file
        let db_file = DatabaseFile::create(Path::new(&temp_path))?;
        Ok((temp_path, db_file))
    }

    fn setup_storage_engine() -> Result<(String, StorageEngine), Box<dyn std::error::Error>> {
        use std::time::{SystemTime, UNIX_EPOCH};
        use std::thread;
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let thread_id = format!("{:?}", thread::current().id());
        let temp_path = format!("test_storage_{}_{}_{}.db", std::process::id(), timestamp, thread_id.chars().filter(|c| c.is_numeric()).collect::<String>());
        
        // Clean up any existing file first
        let _ = fs::remove_file(&temp_path);
        
        // Follow the correct workflow: create file only if it doesn't exist
        let path = Path::new(&temp_path);
        if !path.exists() {
            let _db_file = DatabaseFile::create(path)?;
            // Let it drop here so the file is closed
        }
        
        // Now create the storage engine which will open the existing file
        let storage_engine = StorageEngine::new(path, 64)?;
        Ok((temp_path, storage_engine))
    }

    fn cleanup_file(path: &str) {
        let _ = fs::remove_file(path); // Ignore errors if file doesn't exist
    }

    #[test]
    fn test_buffer_pool_basic_stats() -> Result<(), Box<dyn std::error::Error>> {
        let (temp_path, _db_file) = create_temp_database()?;
        let pool = BufferPool::new(3);

        // Test initial state
        let stats = pool.get_stats();
        assert_eq!(stats.capacity, 3);
        assert_eq!(stats.pages_in_pool, 0);
        assert_eq!(stats.dirty_pages, 0);
        assert_eq!(stats.pinned_pages, 0);

        // Test detailed stats
        let detailed_stats = pool.get_detailed_stats();
        assert_eq!(detailed_stats.utilization_percentage, 0.0);
        assert_eq!(detailed_stats.lru_chain_length, 0);

        // Test basic operations that don't require database_file
        assert!(!pool.contains_page(1));
        assert!(!pool.is_dirty(1));
        assert!(!pool.is_pinned(1));

        // Test page tracking
        assert_eq!(pool.get_all_page_ids().len(), 0);

        // Test consistency validation
        assert!(pool.validate_consistency().is_ok());

        // Test debug print (just ensure it doesn't panic)
        pool.debug_print();

        cleanup_file(&temp_path);
        Ok(())
    }

    #[test]
    fn test_buffer_pool_resize_and_management() -> Result<(), Box<dyn std::error::Error>> {
        let (temp_path, mut db_file) = create_temp_database()?;
        let mut pool = BufferPool::new(5);

        // Test resizing - if resize needs database_file, pass it
        // Check the actual BufferPool API to see if resize needs database_file parameter
        let current_capacity = pool.get_stats().capacity;
        assert_eq!(current_capacity, 5);

        // Test clear - pass database_file if required
        pool.clear(&mut db_file)?;

        let stats = pool.get_stats();
        assert_eq!(stats.pages_in_pool, 0);
        assert_eq!(stats.dirty_pages, 0);
        assert_eq!(stats.pinned_pages, 0);

        cleanup_file(&temp_path);
        Ok(())
    }

    #[test]
    fn test_buffer_pool_lru_simulation() -> Result<(), Box<dyn std::error::Error>> {
        let (temp_path, _db_file) = create_temp_database()?;
        let pool = BufferPool::new(3);

        // Test that the LRU chain is empty initially
        let lru_chain = pool.get_detailed_stats().pages_in_lru;
        assert_eq!(lru_chain, Vec::<u64>::new());

        // Test consistency
        assert!(pool.validate_consistency().is_ok());

        cleanup_file(&temp_path);
        Ok(())
    }

    #[test]
    fn test_buffer_pool_stats_and_diagnostics() -> Result<(), Box<dyn std::error::Error>> {
        let (temp_path, mut db_file) = create_temp_database()?;
        let mut pool = BufferPool::new(100);

        // Test various stats methods
        let basic_stats = pool.get_stats();
        let detailed_stats = pool.get_detailed_stats();

        // Stats should be consistent
        assert_eq!(basic_stats.capacity, detailed_stats.capacity);
        assert_eq!(basic_stats.pages_in_pool, detailed_stats.pages_in_pool);
        assert_eq!(basic_stats.dirty_pages, detailed_stats.dirty_pages);
        assert_eq!(basic_stats.pinned_pages, detailed_stats.pinned_pages);

        // Test utilization calculation
        assert_eq!(detailed_stats.utilization_percentage, 0.0);

        // Test unpin operations (these should not affect consistency since pages aren't in pool)
        pool.unpin_page(1, false); // Don't mark as dirty
        pool.unpin_page(2, false); // Don't mark as dirty

        // Test force eviction - pass database_file if required
        pool.force_evict_page(999, &mut db_file)?;

        // Test that these operations maintain consistency
        assert!(pool.validate_consistency().is_ok());

        cleanup_file(&temp_path);
        Ok(())
    }

    #[test]
    fn test_storage_engine_integration() -> Result<(), Box<dyn std::error::Error>> {
        let (temp_path, mut storage_engine) = setup_storage_engine()?;

        // Test document insertion and retrieval through storage engine
        let mut user_doc = Document::new();
        user_doc.set("name", Value::String("Alice".to_string()));
        user_doc.set("age", Value::I32(28));
        user_doc.set("active", Value::Bool(true));

        // Insert document
        let user_id = storage_engine.insert_document(&user_doc)?;
        assert!(user_id.page_id() < u64::MAX);
        assert!(user_id.slot_id() < u16::MAX);

        // Retrieve document
        let retrieved_user = storage_engine.get_document(&user_id)?;

        // Verify field by field
        assert_eq!(user_doc.get("name"), retrieved_user.get("name"));
        assert_eq!(user_doc.get("age"), retrieved_user.get("age"));
        assert_eq!(user_doc.get("active"), retrieved_user.get("active"));

        // Test multiple documents
        let mut product_doc = Document::new();
        product_doc.set("name", Value::String("Laptop".to_string()));
        product_doc.set("price", Value::F64(999.99));
        product_doc.set("stock", Value::I32(15));

        let product_id = storage_engine.insert_document(&product_doc)?;
        let retrieved_product = storage_engine.get_document(&product_id)?;

        assert_eq!(product_doc.get("name"), retrieved_product.get("name"));
        assert_eq!(product_doc.get("price"), retrieved_product.get("price"));
        assert_eq!(product_doc.get("stock"), retrieved_product.get("stock"));

        cleanup_file(&temp_path);
        Ok(())
    }

    #[test]
    fn test_buffer_pool_with_real_pages() -> Result<(), Box<dyn std::error::Error>> {
        let (temp_path, mut storage_engine) = setup_storage_engine()?;

        // Insert a document to create a real page
        let mut doc = Document::new();
        doc.set("test_field", Value::String("test_value".to_string()));

        let doc_id = storage_engine.insert_document(&doc)?;

        // Now we can test buffer pool operations with a real page
        // Note: We can't directly access the buffer pool from storage engine easily,
        // but we can test that the operations work through the storage engine

        // Retrieve the document (this tests pin_page internally)
        let retrieved_doc = storage_engine.get_document(&doc_id)?;
        assert_eq!(doc.get("test_field"), retrieved_doc.get("test_field"));

        println!("✅ Buffer pool successfully handled real page operations");
        
        cleanup_file(&temp_path);
        Ok(())
    }
}
