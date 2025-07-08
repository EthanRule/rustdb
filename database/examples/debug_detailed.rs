use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn print_slot_info(page: &Page, label: &str) {
    println!("\n=== {} ===", label);
    
    // Read header manually (slot directory is at the end of the page)
    let data = page.to_bytes();
    let slot_dir_header_offset = 8192 - 4; // Last 4 bytes for header
    let slot_count = u16::from_le_bytes([data[slot_dir_header_offset], data[slot_dir_header_offset + 1]]);
    let free_space_offset = u16::from_le_bytes([data[slot_dir_header_offset + 2], data[slot_dir_header_offset + 3]]);
    
    println!("Header: slot_count={}, free_space_offset={}", slot_count, free_space_offset);
    
    // Read each slot entry manually
    if slot_count > 0 && slot_count < 1000 { // Safety check
        let slot_dir_start = 8192 - 4 - (slot_count as usize * 4);
        for slot_id in 0..slot_count {
            let slot_offset = slot_dir_start + (slot_id as usize * 4);
            let offset = u16::from_le_bytes([data[slot_offset], data[slot_offset + 1]]);
            let length = u16::from_le_bytes([data[slot_offset + 2], data[slot_offset + 3]]);
            
            println!("Slot {}: offset={}, length={}", slot_id, offset, length);
        }
    }
    
    println!("Document count: {}", PageLayout::get_document_count(page).unwrap());
}

fn main() {
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    
    print_slot_info(&page, "After initialization");
    
    let doc1 = b"Document 1";
    let slot1 = PageLayout::insert_document(&mut page, doc1).unwrap();
    println!("\nInserted doc1, got slot: {}", slot1);
    print_slot_info(&page, "After inserting doc1");
    
    let doc2 = b"Document 2";
    let slot2 = PageLayout::insert_document(&mut page, doc2).unwrap();
    println!("\nInserted doc2, got slot: {}", slot2);
    print_slot_info(&page, "After inserting doc2");
    
    let doc3 = b"Document 3";
    let slot3 = PageLayout::insert_document(&mut page, doc3).unwrap();
    println!("\nInserted doc3, got slot: {}", slot3);
    print_slot_info(&page, "After inserting doc3");
}
