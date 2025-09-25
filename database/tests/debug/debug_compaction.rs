/// Debug version of the failing test to understand the issue
use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn create_test_page() -> Page {
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    page
}

#[test]
fn debug_page_state_transitions() {
    let mut page = create_test_page();
    
    // Test empty -> full -> fragmented -> compacted -> full cycle
    let mut phase_documents = Vec::new();
    
    // Phase 1: Fill page (EXACTLY like the failing test)
    for i in 0..50 {
        let doc = vec![b'A' + (i % 26) as u8; 50 + (i % 50)];
        if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
            phase_documents.push((slot, doc));
            println!("Inserted doc {} into slot {}", i, slot);
        } else {
            println!("Failed to insert doc {} - page full", i);
            break;
        }
    }
    
    let full_count = PageLayout::get_document_count(&page).expect("Failed to get count");
    let _full_utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
    println!("After Phase 1: {} documents", full_count);
    
    // Phase 2: Create fragmentation (EXACTLY like the failing test)
    let to_delete = phase_documents.len() / 3;
    println!("Phase 2: Will delete {} documents (every 3rd)", to_delete);
    
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
    
    let fragmented_count = PageLayout::get_document_count(&page).expect("Failed to get count");
    assert!(fragmented_count < full_count, "Count should decrease after deletions");
    println!("After Phase 2: {} documents (was {})", fragmented_count, full_count);
    
    // Phase 3: Compact (EXACTLY like the failing test)
    println!("Phase 3: Compacting...");
    PageLayout::compact_page(&mut page).expect("Failed to compact");
    
    let compacted_count = PageLayout::get_document_count(&page).expect("Failed to get count");
    assert_eq!(compacted_count, fragmented_count, "Count should remain same after compaction");
    
    let compacted_utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
    assert!(compacted_utilization >= 0.0 && compacted_utilization <= 100.0);
    println!("After Phase 3: {} documents, utilization: {:.2}%", compacted_count, compacted_utilization);
    
    // Phase 4: Verify remaining documents are intact (EXACTLY like the failing test)
    println!("Phase 4: Verifying document integrity with EXACT failing test logic...");
    for (i, (slot, expected_doc)) in phase_documents.iter().enumerate() {
        let result = PageLayout::get_document(&page, *slot);
        
        if i % 3 == 0 {
            // This document was deleted
            println!("Checking deleted doc at index {} (slot {}): {:?}", i, slot, result.as_ref().map(|_| "OK").map_err(|e| format!("{:?}", e)));
            
            // Check if this index was actually deleted
            let should_be_deleted = i < 16 * 3; // Only first 16 * 3 = 48 were actually deleted
            if !should_be_deleted {
                println!("Note: Index {} was NOT actually deleted (i={}, 16*3={})", i, i, 16*3);
            }
            
            if result.is_ok() {
                if should_be_deleted {
                    println!("ERROR: Deleted document at index {} (slot {}) is still retrievable after compaction!", i, slot);
                    println!("Expected to be deleted, but got document of length {}", result.unwrap().len());
                    println!("This is the bug!");
                    panic!("Deleted document should not be retrievable");
                } else {
                    println!("OK: Document at index {} (slot {}) was never deleted, so it's fine that it's still there", i, slot);
                }
            } else {
                println!("✓ Deleted document at index {} (slot {}) is correctly not retrievable", i, slot);
            }
        } else {
            // This document should still exist
            match result {
                Ok(retrieved) => {
                    if retrieved == *expected_doc {
                        println!("✓ Active doc at index {} (slot {}) is correct", i, slot);
                    } else {
                        println!("ERROR: Document at index {} (slot {}) was corrupted!", i, slot);
                        println!("Expected doc of length {}, got doc of length {}", expected_doc.len(), retrieved.len());
                        panic!("Document corrupted during state transitions");
                    }
                }
                Err(e) => {
                    println!("ERROR: Active document at index {} (slot {}) is not retrievable: {:?}", i, slot, e);
                    panic!("Remaining document should be retrievable");
                }
            }
        }
    }
    
    println!("Phase 4 completed successfully!");
}
