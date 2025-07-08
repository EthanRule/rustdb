use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn main() {
    // Create a new page
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    
    println!("=== Test 1: Basic insertion ===");
    let doc1 = b"Document 1";
    let slot1 = PageLayout::insert_document(&mut page, doc1).unwrap();
    println!("Inserted doc1, got slot: {}", slot1);
    println!("Document count: {}", PageLayout::get_document_count(&page).unwrap());
    
    let doc2 = b"Document 2";
    let slot2 = PageLayout::insert_document(&mut page, doc2).unwrap();
    println!("Inserted doc2, got slot: {}", slot2);
    println!("Document count: {}", PageLayout::get_document_count(&page).unwrap());
    
    let doc3 = b"Document 3";
    let slot3 = PageLayout::insert_document(&mut page, doc3).unwrap();
    println!("Inserted doc3, got slot: {}", slot3);
    println!("Document count: {}", PageLayout::get_document_count(&page).unwrap());
    
    println!("\n=== Test 2: Reading back ===");
    match PageLayout::get_document(&page, slot1) {
        Ok(data) => println!("slot{}: {:?}", slot1, String::from_utf8_lossy(&data)),
        Err(e) => println!("slot{}: ERROR - {}", slot1, e),
    }
    match PageLayout::get_document(&page, slot2) {
        Ok(data) => println!("slot{}: {:?}", slot2, String::from_utf8_lossy(&data)),
        Err(e) => println!("slot{}: ERROR - {}", slot2, e),
    }
    match PageLayout::get_document(&page, slot3) {
        Ok(data) => println!("slot{}: {:?}", slot3, String::from_utf8_lossy(&data)),
        Err(e) => println!("slot{}: ERROR - {}", slot3, e),
    }
}
