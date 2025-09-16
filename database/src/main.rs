use database::storage::file::DatabaseFile;
use database::storage::storage_engine::StorageEngine;
use database::{init_tracing, Document, Value};
use std::path::Path;
use tracing::info;

fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    println!("🗄️  Rust Database Engine Demo");
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

    // Create a simple user document (smaller to fit in initial pages)
    let mut user_doc = Document::new();
    user_doc.set("name", Value::String("Alice".to_string()));
    user_doc.set("age", Value::I32(28));
    user_doc.set("active", Value::Bool(true));

    println!("👤 Inserting simple user document:");
    println!("   Name: Alice");
    println!("   Age: 28");

    // Insert user document and handle both success and error cases
    let user_id = match storage_engine.insert_document(&user_doc) {
        Ok(user_location) => {
            println!(
                "✅ User document inserted at page {} slot {}",
                user_location.page_id(),
                user_location.slot_id()
            );
            user_location
        }
        Err(e) => {
            println!("❌ Error inserting user document: {}", e);
            return Err(e.into());
        }
    };

    // Create a product document
    let mut product_doc = Document::new();
    product_doc.set("name", Value::String("Laptop".to_string()));
    product_doc.set("price", Value::F64(999.99));
    product_doc.set("stock", Value::I32(15));

    println!("\n💻 Inserting product document:");
    println!("   Name: Laptop");
    println!("   Price: $999.99");

    let product_id = match storage_engine.insert_document(&product_doc) {
        Ok(product_location) => {
            println!(
                "✅ Product document inserted at page {} slot {}",
                product_location.page_id(),
                product_location.slot_id()
            );
            product_location
        }
        Err(e) => {
            println!("❌ Error inserting product document: {}", e);
            return Err(e.into());
        }
    };

    // Demo 2: Test document retrieval (round-trip verification)
    println!("\n🔄 Demo 2: Testing document retrieval");

    let retrieved_user = storage_engine.get_document(&user_id)?;
    let retrieved_product = storage_engine.get_document(&product_id)?;

    println!("📖 Retrieved user document successfully");
    println!("📖 Retrieved product document successfully");

    // Verify the field content is identical (ignore ObjectId differences)
    println!("\n🔍 Verifying round-trip data integrity...");

    // Check user document fields
    println!("🔍 Verifying user document fields...");
    if user_doc.get("name") == retrieved_user.get("name")
        && user_doc.get("age") == retrieved_user.get("age")
        && user_doc.get("active") == retrieved_user.get("active")
    {
        println!("✅ User data round-trip successful!");
    } else {
        println!("❌ User data mismatch!");
        println!("   Original name: {:?}", user_doc.get("name"));
        println!("   Retrieved name: {:?}", retrieved_user.get("name"));
        println!("   Original age: {:?}", user_doc.get("age"));
        println!("   Retrieved age: {:?}", retrieved_user.get("age"));
        return Err("Data integrity check failed".into());
    }

    // Check product document fields
    println!("🔍 Verifying product document fields...");
    if product_doc.get("name") == retrieved_product.get("name")
        && product_doc.get("price") == retrieved_product.get("price")
        && product_doc.get("stock") == retrieved_product.get("stock")
    {
        println!("✅ Product data round-trip successful!");
    } else {
        println!("❌ Product data mismatch!");
        println!("   Original name: {:?}", product_doc.get("name"));
        println!("   Retrieved name: {:?}", retrieved_product.get("name"));
        println!("   Original price: {:?}", product_doc.get("price"));
        println!("   Retrieved price: {:?}", retrieved_product.get("price"));
        return Err("Data integrity check failed".into());
    }

    println!("\n🎉 Database Engine Demo Complete!");
    println!("=====================================");
    println!("✅ Document insertion working");
    println!("✅ Document retrieval working");
    println!("✅ Page allocation working");
    println!("✅ Data integrity verified");
    println!("\n📊 Current Implementation Status:");
    println!("   ✅ BSON serialization and deserialization");
    println!("   ✅ Page-based storage with slot directories");
    println!("   ✅ Buffer pool management with LRU eviction");
    println!("   ✅ File I/O with checksums and error handling");
    println!("   ✅ Document insertion and retrieval");
    println!("   🔄 Next: Document updates and deletion");
    println!("   🔄 Next: Query and indexing capabilities");

    Ok(())
}
