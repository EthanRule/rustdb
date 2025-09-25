use database::storage::file::DatabaseFile;
use database::storage::storage_engine::StorageEngine;
use std::path::Path;
use std::fs;

#[test]
fn test_minimal_file_lock_debug() {
    println!("🔍 Starting minimal file lock debug test");
    
    let test_file = "debug_minimal.db";
    let _ = fs::remove_file(test_file);
    
    println!("📁 Step 1: Creating DatabaseFile");
    let db_file_result = DatabaseFile::create(Path::new(test_file));
    match &db_file_result {
        Ok(_) => println!("✅ DatabaseFile created successfully"),
        Err(e) => {
            println!("❌ DatabaseFile creation failed: {}", e);
            return;
        }
    }
    
    // Explicitly drop the DatabaseFile
    println!("📁 Step 2: Dropping DatabaseFile");
    drop(db_file_result);
    
    println!("📁 Step 3: Creating StorageEngine");
    let storage_result = StorageEngine::new(Path::new(test_file), 64);
    match &storage_result {
        Ok(_) => println!("✅ StorageEngine created successfully"),
        Err(e) => {
            println!("❌ StorageEngine creation failed: {}", e);
            println!("🐛 This is where the file lock occurs!");
            
            // Try to clean up and see if file is still locked
            println!("📁 Step 4: Attempting file cleanup");
            let remove_result = fs::remove_file(test_file);
            match remove_result {
                Ok(_) => println!("✅ File removed successfully"),
                Err(e) => println!("❌ File removal failed (still locked): {}", e),
            }
            return;
        }
    }
    
    println!("📁 Step 4: Dropping StorageEngine");
    drop(storage_result);
    
    println!("📁 Step 5: Final cleanup");
    let _ = fs::remove_file(test_file);
    println!("✅ Test completed successfully");
}

#[test]
fn test_database_file_only() {
    println!("🔍 Testing DatabaseFile creation/destruction only");
    
    let test_file = "debug_dbfile_only.db";
    let _ = fs::remove_file(test_file);
    
    {
        println!("📁 Creating DatabaseFile in scope");
        let _db_file = DatabaseFile::create(Path::new(test_file)).expect("Failed to create DatabaseFile");
        println!("✅ DatabaseFile created, about to go out of scope");
    }
    
    println!("📁 DatabaseFile should be dropped now");
    
    // Try to create another one
    let result = DatabaseFile::create(Path::new(test_file));
    match result {
        Ok(_) => println!("✅ Second DatabaseFile created successfully"),
        Err(e) => println!("❌ Second DatabaseFile creation failed: {}", e),
    }
    
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_storage_engine_only() {
    println!("🔍 Testing StorageEngine creation after manual file creation");
    
    let test_file = "debug_storage_only.db";
    let _ = fs::remove_file(test_file);
    
    // Create database file and immediately drop it
    {
        let _db_file = DatabaseFile::create(Path::new(test_file)).expect("Failed to create DatabaseFile");
    }
    
    // Now try to create StorageEngine
    println!("📁 Creating StorageEngine");
    let result = StorageEngine::new(Path::new(test_file), 64);
    match result {
        Ok(_) => println!("✅ StorageEngine created successfully"),
        Err(e) => println!("❌ StorageEngine creation failed: {}", e),
    }
    
    let _ = fs::remove_file(test_file);
}