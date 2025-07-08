/// Test to reproduce the compaction bug
use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn create_test_page() -> Page {
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    page
}

#[test]
fn test_compaction_preserves_tombstones() {
    let mut page = create_test_page();
    
    // Insert documents exactly like the failing test
    let mut phase_documents = Vec::new();
    for i in 0..10 { // Smaller number for easier debugging
        let doc = vec![b'A' + (i % 26) as u8; 50 + (i % 50)];
        if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
            phase_documents.push((slot, doc));
            println!("Inserted doc {} into slot {}", i, slot);
        } else {
            break;
        }
    }
    
    println!("Inserted {} documents", phase_documents.len());
    
    // Delete every third document exactly like the failing test
    let to_delete = phase_documents.len() / 3;
    let mut deleted_indices = Vec::new();
    for i in 0..to_delete {
        let idx = i * 3; // Delete every third document
        if idx < phase_documents.len() {
            let (slot, _) = &phase_documents[idx];
            PageLayout::delete_document(&mut page, *slot).expect("Failed to delete");
            deleted_indices.push(idx);
            println!("Deleted document at index {} (slot {})", idx, slot);
        }
    }
    
    // Verify state before compaction
    for (i, (slot, _)) in phase_documents.iter().enumerate() {
        let result = PageLayout::get_document(&page, *slot);
        if deleted_indices.contains(&i) {
            assert!(result.is_err(), "Document at index {} (slot {}) should be deleted before compaction", i, slot);
            println!("✓ Document at index {} (slot {}) is correctly deleted before compaction", i, slot);
        } else {
            assert!(result.is_ok(), "Document at index {} (slot {}) should be accessible before compaction", i, slot);
            println!("✓ Document at index {} (slot {}) is correctly accessible before compaction", i, slot);
        }
    }
    
    // Compact the page
    println!("Compacting page...");
    PageLayout::compact_page(&mut page).expect("Failed to compact page");
    
    // Verify state after compaction - this is where the bug manifests
    for (i, (slot, expected_doc)) in phase_documents.iter().enumerate() {
        let result = PageLayout::get_document(&page, *slot);
        if deleted_indices.contains(&i) {
            // This document was deleted
            println!("Checking deleted document at index {} (slot {}): {:?}", i, slot, result);
            assert!(result.is_err(), "Document at index {} (slot {}) should still be deleted after compaction", i, slot);
        } else {
            // This document should still exist
            println!("Checking active document at index {} (slot {}): OK", i, slot);
            let retrieved = result.expect(&format!("Document at index {} (slot {}) should still be accessible after compaction", i, slot));
            assert_eq!(retrieved, *expected_doc, "Document at index {} (slot {}) corrupted during compaction", i, slot);
        }
    }
    
    println!("Test completed successfully!");
}
