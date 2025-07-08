use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn print_full_debug(page: &Page, label: &str) {
    println!("\n=== {} ===", label);
    
    let data = page.to_bytes();
    let slot_dir_header_offset = 8192 - 4;
    let slot_count = u16::from_le_bytes([data[slot_dir_header_offset], data[slot_dir_header_offset + 1]]);
    let free_space_offset = u16::from_le_bytes([data[slot_dir_header_offset + 2], data[slot_dir_header_offset + 3]]);
    
    println!("Header: slot_count={}, free_space_offset={}", slot_count, free_space_offset);
    
    if slot_count > 0 && slot_count < 10 {
        let slot_dir_start = 8192 - 4 - (slot_count as usize * 4);
        for slot_id in 0..slot_count {
            let slot_offset = slot_dir_start + (slot_id as usize * 4);
            let offset = u16::from_le_bytes([data[slot_offset], data[slot_offset + 1]]);
            let length = u16::from_le_bytes([data[slot_offset + 2], data[slot_offset + 3]]);
            
            println!("Slot {}: offset={}, length={}", slot_id, offset, length);
            
            // Print the actual document data
            if offset > 0 && length > 0 && (offset as usize + length as usize) <= data.len() {
                let doc_data = &data[offset as usize..(offset as usize + length as usize)];
                println!("  Data: {:?}", String::from_utf8_lossy(doc_data));
            }
        }
    }
    
    println!("Document count via API: {}", PageLayout::get_document_count(page).unwrap());
}

fn main() {
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    
    print_full_debug(&page, "After initialization");
    
    println!("\n>>> Inserting Document 1");
    let doc1 = b"Document 1";
    let slot1 = PageLayout::insert_document(&mut page, doc1).unwrap();
    println!("Got slot: {}", slot1);
    print_full_debug(&page, "After inserting doc1");
    
    println!("\n>>> Inserting Document 2");
    let doc2 = b"Document 2";
    let slot2 = PageLayout::insert_document(&mut page, doc2).unwrap();
    println!("Got slot: {}", slot2);
    print_full_debug(&page, "After inserting doc2");
    
    println!("\n>>> Inserting Document 3");
    let doc3 = b"Document 3";
    let slot3 = PageLayout::insert_document(&mut page, doc3).unwrap();
    println!("Got slot: {}", slot3);
    print_full_debug(&page, "After inserting doc3");
    
    println!("\n>>> Reading back using API");
    for slot_id in 0..3 {
        match PageLayout::get_document(&page, slot_id) {
            Ok(data) => println!("API slot {}: {:?}", slot_id, String::from_utf8_lossy(&data)),
            Err(e) => println!("API slot {}: ERROR - {}", slot_id, e),
        }
    }
}
