use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn main() {
    println!("PAGE_SIZE = 8192");
    println!("SLOT_DIRECTORY_OFFSET = 8192 - 4 = 8188");
    println!("SLOT_SIZE = 4");
    
    println!("\nSlot directory calculations:");
    for slot_count in 0..5 {
        let slot_dir_start = 8188 - (slot_count * 4);
        println!("slot_count={}: slot_dir_start={}", slot_count, slot_dir_start);
        
        for slot_id in 0..slot_count {
            let slot_offset = slot_dir_start + (slot_id * 4);
            println!("  slot {}: offset {}", slot_id, slot_offset);
        }
    }
    
    println!("\nLet's trace the actual insertion:");
    
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    
    println!("1. Insert doc1:");
    let doc1 = b"Document 1";
    let slot1 = PageLayout::insert_document(&mut page, doc1).unwrap();
    println!("   slot1 = {}", slot1);
    
    println!("2. Insert doc2:");
    let doc2 = b"Document 2";  
    let slot2 = PageLayout::insert_document(&mut page, doc2).unwrap();
    println!("   slot2 = {}", slot2);
    
    // Manually check where each slot should be stored
    println!("\nManual slot directory check:");
    let data = page.to_bytes();
    
    // For slot_count=1: slot_dir_start = 8188 - 4 = 8184
    // slot 0 should be at 8184
    println!("slot 0 (8184): offset={}, length={}", 
        u16::from_le_bytes([data[8184], data[8185]]),
        u16::from_le_bytes([data[8186], data[8187]]));
    
    // For slot_count=2: slot_dir_start = 8188 - 8 = 8180  
    // slot 0 should be at 8180, slot 1 at 8184
    println!("slot 0 (8180): offset={}, length={}", 
        u16::from_le_bytes([data[8180], data[8181]]),
        u16::from_le_bytes([data[8182], data[8183]]));
    println!("slot 1 (8184): offset={}, length={}", 
        u16::from_le_bytes([data[8184], data[8185]]),
        u16::from_le_bytes([data[8186], data[8187]]));
}
