use database::storage::page_layout::PageLayout;
use database::storage::page::{Page, PageType};

#[test]
fn test_simple_insert_debug() {
    // Create and manually initialize a page
    let mut page = Page::new(1, PageType::Data);
    
    // Manually set up the slot directory at the end of the page
    // Page is 8192 bytes, slot directory header is at offset 8184 (8192 - 8)
    let data = unsafe { &mut *((&mut page) as *mut Page as *mut [u8; 8192]) };
    
    // Initialize slot directory header: slot_count=0, free_space_offset=16
    data[8184] = 0; // slot_count low byte
    data[8185] = 0; // slot_count high byte  
    data[8186] = 16; // free_space_offset low byte (header size)
    data[8187] = 0; // free_space_offset high byte
    
    println!("Page manually initialized");
    
    // Test document data
    let doc = b"test";
    
    // Try to insert
    println!("Attempting to insert document...");
    match PageLayout::insert_document(&mut page, doc) {
        Ok(slot_id) => {
            println!("Insert successful, slot_id: {}", slot_id);
            
            // Try to retrieve
            match PageLayout::get_document(&page, slot_id) {
                Ok(data) => {
                    println!("Retrieved data: {:?}", std::str::from_utf8(&data));
                    assert_eq!(data, doc);
                }
                Err(e) => {
                    println!("Get failed: {:?}", e);
                    panic!("Get failed");
                }
            }
        }
        Err(e) => {
            println!("Insert failed: {:?}", e);
            panic!("Insert failed");
        }
    }
}
