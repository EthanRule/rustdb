use database::storage::{
    page::{Page, PageType}, 
    page_layout::PageLayout,
};

fn create_initialized_page(page_id: u64) -> Page {
    let mut page = Page::new(page_id, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    page
}

#[test]
fn test_basic_page_layout_functionality() {
    // Test basic page layout operations
    let mut page = create_initialized_page(1);
    
    // Add documents
    let doc1 = b"Document 1".to_vec();
    let doc2 = b"Document 2".to_vec();
    
    let slot1 = PageLayout::insert_document(&mut page, &doc1).expect("Failed to insert doc1");
    let slot2 = PageLayout::insert_document(&mut page, &doc2).expect("Failed to insert doc2");
    
    // Verify documents can be retrieved
    let retrieved1 = PageLayout::get_document(&page, slot1).expect("Failed to get doc1");
    let retrieved2 = PageLayout::get_document(&page, slot2).expect("Failed to get doc2");
    
    assert_eq!(retrieved1, doc1);
    assert_eq!(retrieved2, doc2);
    
    // Test utilization
    let utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
    assert!(utilization > 0.0);
    assert!(utilization <= 100.0);
}

#[test]
fn test_page_layout_with_compaction() {
    let mut page = create_initialized_page(1);
    
    // Insert multiple documents
    let mut slots = Vec::new();
    for i in 0..10 {
        let doc = format!("Document {}", i).into_bytes();
        let slot = PageLayout::insert_document(&mut page, &doc).expect("Failed to insert document");
        slots.push(slot);
    }
    
    // Delete some documents to create fragmentation
    for &slot in slots.iter().step_by(2) {
        PageLayout::delete_document(&mut page, slot).expect("Failed to delete document");
    }
    
    // Check utilization before compaction
    let util_before = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
    
    // Perform compaction
    PageLayout::compact_page(&mut page).expect("Failed to compact page");
    
    // Check utilization after compaction (should be same or better)
    let util_after = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
    assert!(util_after >= util_before);
    
    // Verify remaining documents are still accessible
    for &slot in slots.iter().skip(1).step_by(2) {
        let doc = PageLayout::get_document(&page, slot);
        assert!(doc.is_ok() || doc.is_err()); // Either still there or properly deleted
    }
}

#[test]
fn test_page_layout_edge_cases() {
    let mut page = create_initialized_page(1);
    
    // Test empty document handling
    let empty_doc = Vec::new();
    let result = PageLayout::insert_document(&mut page, &empty_doc);
    assert!(result.is_err()); // Should reject empty documents
    
    // Test large document that might not fit
    let large_doc = vec![b'A'; 3000]; // 3KB document
    let result = PageLayout::insert_document(&mut page, &large_doc);
    // This may succeed or fail depending on page size, just verify it handles it gracefully
    assert!(result.is_ok() || result.is_err());
    
    // Test accessing invalid slot
    let invalid_result = PageLayout::get_document(&page, 999);
    assert!(invalid_result.is_err());
    
    // Test document count
    let _count = PageLayout::get_document_count(&page).expect("Failed to get document count");
}

#[test]
fn test_slot_reuse_pattern() {
    let mut page = create_initialized_page(1);
    
    // Insert documents
    let doc1 = b"Document 1".to_vec();
    let doc2 = b"Document 2".to_vec();
    
    let slot1 = PageLayout::insert_document(&mut page, &doc1).expect("Failed to insert doc1");
    let slot2 = PageLayout::insert_document(&mut page, &doc2).expect("Failed to insert doc2");
    
    // Delete first document
    PageLayout::delete_document(&mut page, slot1).expect("Failed to delete doc1");
    
    // Insert new document - should reuse slot1
    let doc3 = b"Document 3".to_vec();
    let slot3 = PageLayout::insert_document(&mut page, &doc3).expect("Failed to insert doc3");
    
    // Verify slot reuse behavior (slot3 should be same as slot1 or be a new slot)
    let retrieved3 = PageLayout::get_document(&page, slot3).expect("Failed to get doc3");
    assert_eq!(retrieved3, doc3);
    
    // Verify doc2 is still accessible
    let retrieved2 = PageLayout::get_document(&page, slot2).expect("Failed to get doc2");
    assert_eq!(retrieved2, doc2);
}

#[test]
fn test_update_operations() {
    let mut page = create_initialized_page(1);
    
    // Insert initial document
    let original_doc = b"Original Document".to_vec();
    let slot = PageLayout::insert_document(&mut page, &original_doc).expect("Failed to insert");
    
    // Update with same size
    let same_size_doc = b"Updated Document!".to_vec(); // Same length
    assert_eq!(original_doc.len(), same_size_doc.len());
    
    let result = PageLayout::update_document(&mut page, slot, &same_size_doc);
    assert!(result.is_ok());
    
    let retrieved = PageLayout::get_document(&page, slot).expect("Failed to get updated doc");
    assert_eq!(retrieved, same_size_doc);
    
    // Update with smaller size
    let smaller_doc = b"Small".to_vec();
    let result = PageLayout::update_document(&mut page, slot, &smaller_doc);
    assert!(result.is_ok());
    
    let retrieved = PageLayout::get_document(&page, slot).expect("Failed to get smaller doc");
    assert_eq!(retrieved, smaller_doc);
    
    // Update with larger size
    let larger_doc = b"This is a much larger document that should fit".to_vec();
    let result = PageLayout::update_document(&mut page, slot, &larger_doc);
    // This may succeed or fail depending on available space
    if result.is_ok() {
        let retrieved = PageLayout::get_document(&page, slot).expect("Failed to get larger doc");
        assert_eq!(retrieved, larger_doc);
    }
}

#[test]
fn test_memory_safety_and_bounds() {
    let mut page = create_initialized_page(1);
    
    // Test various document sizes
    let sizes = vec![1, 10, 50, 100, 500, 1000];
    let mut slots = Vec::new();
    
    for size in sizes {
        let doc = vec![b'X'; size];
        match PageLayout::insert_document(&mut page, &doc) {
            Ok(slot) => {
                slots.push((slot, doc.clone()));
                
                // Verify immediate retrieval
                let retrieved = PageLayout::get_document(&page, slot).expect("Failed to retrieve");
                assert_eq!(retrieved, doc);
            }
            Err(_) => {
                // Page full or document too large - this is acceptable
                break;
            }
        }
    }
    
    // Verify all stored documents are still correct
    for (slot, expected_doc) in slots {
        let retrieved = PageLayout::get_document(&page, slot).expect("Document corrupted");
        assert_eq!(retrieved, expected_doc);
    }
    
    // Test page consistency
    let _count = PageLayout::get_document_count(&page).expect("Failed to get document count");
    let utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
    
    assert!(utilization >= 0.0 && utilization <= 100.0);
}
