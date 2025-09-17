use database::storage::file::DatabaseFile;
use database::storage::storage_engine::StorageEngine;
use database::{init_tracing, Document, Value};
use std::path::Path;
use tracing::info;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    println!("🗄️  Rus  t Database Engine Demo");
    println!("==============================");

    info!("Starting database engine demonstration");

    // Create a new database file for our demo
    let db_path = "demo_database.db";
    println!("\n📂 Creating database file: {}", db_path);

    // First create the database file if it doesn't exist
    let db_path_obj = Path::new(db_path);
    if !db_path_obj.exists() {
        let _db_file = DatabaseFile::create(db_path_obj)?;
        println!("✅ New database file created");
    }

    // Initialize storage engine with buffer pool size
    let mut storage_engine = StorageEngine::new(db_path_obj, 64)?;
    println!("✅ Storage engine initialized successfully");

    // Demo 1: Create and insert various document types
    println!("\n📝 Demo 1: Creating and inserting documents");
    println!("✅ Page allocation is working! Documents will be stored in allocated pages");

    // Create a simple user document
    let mut user_doc = Document::new();
    user_doc.set("name", Value::String("Alice".to_string()));
    user_doc.set("age", Value::I32(28));
    user_doc.set("active", Value::Bool(true));
    user_doc.set("email", Value::String("alice@example.com".to_string()));

    println!("👤 Creating user document:");
    println!("   Name: Alice");
    println!("   Age: 28");
    println!("   Email: alice@example.com");

    let user_id = storage_engine.insert_document(&user_doc)?;
    println!("✅ User document inserted at page {} slot {}", 
             user_id.page_id(), user_id.slot_id());

    // Create a product document
    let mut product_doc = Document::new();
    product_doc.set("name", Value::String("Laptop".to_string()));
    product_doc.set("price", Value::F64(999.99));
    product_doc.set("stock", Value::I32(15));
    product_doc.set("category", Value::String("Electronics".to_string()));

    println!("\n💻 Creating product document:");
    println!("   Name: Laptop");
    println!("   Price: $999.99");
    println!("   Stock: 15");
    println!("   Category: Electronics");

    let product_id = storage_engine.insert_document(&product_doc)?;
    println!("✅ Product document inserted at page {} slot {}", 
             product_id.page_id(), product_id.slot_id());

    // Demo 2: Retrieve and display documents
    println!("\n� Demo 2: Retrieving documents");

    let retrieved_user = storage_engine.get_document(&user_id)?;
    println!("📖 Retrieved user document:");
    println!("   Name: {:?}", retrieved_user.get("name"));
    println!("   Age: {:?}", retrieved_user.get("age"));
    println!("   Active: {:?}", retrieved_user.get("active"));
    println!("   Email: {:?}", retrieved_user.get("email"));

    let retrieved_product = storage_engine.get_document(&product_id)?;
    println!("\n📖 Retrieved product document:");
    println!("   Name: {:?}", retrieved_product.get("name"));
    println!("   Price: {:?}", retrieved_product.get("price"));
    println!("   Stock: {:?}", retrieved_product.get("stock"));
    println!("   Category: {:?}", retrieved_product.get("category"));

    // Demo 3: Show database features
    println!("\n📊 Demo 3: Database Engine Features");
    println!("====================================");
    println!("✅ BSON Document Serialization");
    println!("   - Supports String, I32, I64, F64, Bool types");
    println!("   - Efficient binary storage format");
    
    println!("\n✅ Page-Based Storage");
    println!("   - 8KB pages with slot directories");
    println!("   - Multiple documents per page");
    println!("   - Page-level checksums for integrity");
    
    println!("\n✅ Buffer Pool Management");
    println!("   - LRU eviction policy");
    println!("   - Configurable buffer size");
    println!("   - Pin/unpin page mechanism");
    
    println!("\n✅ File Management");
    println!("   - Database file headers");
    println!("   - Page allocation and tracking");
    println!("   - Cross-platform file I/O");

    println!("\n🚀 Demo 4: Implementation Status");
    println!("=================================");
    println!("Completed:");
    println!("  ✅ Document insertion and retrieval");
    println!("  ✅ BSON serialization/deserialization");
    println!("  ✅ Page-based storage architecture");
    println!("  ✅ Buffer pool with LRU caching");
    println!("  ✅ File I/O with error handling");
    println!("  ✅ Memory management and safety");
    
    println!("\nNext Features:");
    println!("  🔄 Document updates and deletion");
    println!("  🔄 Query processing and filtering");
    println!("  🔄 Index structures for fast lookups");
    println!("  🔄 Transaction support");

    println!("\n🎯 Testing");
    println!("==========");
    println!("Run comprehensive tests with:");
    println!("  cargo test");
    println!("\nRun specific test suites:");
    println!("  cargo test storage_engine_roundtrip_test");
    println!("  cargo test page_layout");
    println!("  cargo test buffer_pool");

    info!("Database demonstration completed successfully!");
    println!("\n✨ Demo completed successfully! ✨");
    println!("🎉 Your database engine has working document storage! 🎉");

    Ok(())
}
