/// Advanced Quality Assurance Tests for Page Layout Manager
/// These tests focus on complex scenarios, edge cases, and potential race conditions
use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn create_test_page() -> Page {
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    page
}

#[cfg(test)]
mod advanced_qa_tests {
    use super::*;

    /// Test extremely complex document lifecycle patterns
    #[test]
    fn test_complex_document_lifecycle() {
        let mut page = create_test_page();
        let mut active_slots = Vec::new();
        
        // Phase 1: Insert documents with various sizes
        for i in 0..20 {
            let size = match i % 4 {
                0 => 10,   // Small documents
                1 => 50,   // Medium documents
                2 => 100,  // Large documents
                3 => 25,   // Variable size
                _ => unreachable!(),
            };
            
            let doc = vec![b'A' + (i as u8) % 26; size];
            if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
                active_slots.push((slot, doc));
            }
        }
        
        // Phase 2: Random updates, deletions, and insertions
        for cycle in 0..10 {
            // Delete some documents
            let to_delete = active_slots.len() / 3;
            for _ in 0..to_delete {
                if !active_slots.is_empty() {
                    let idx = cycle % active_slots.len();
                    let (slot, _) = active_slots.remove(idx);
                    PageLayout::delete_document(&mut page, slot).expect("Failed to delete");
                }
            }
            
            // Update some documents
            let to_update = active_slots.len() / 2;
            for i in 0..to_update {
                if i < active_slots.len() {
                    let (slot, old_doc) = &mut active_slots[i];
                    let new_size = if cycle % 2 == 0 { 
                        old_doc.len() / 2 + 1  // Shrink
                    } else { 
                        old_doc.len() * 2      // Grow
                    };
                    
                    let new_doc = vec![b'X'; new_size];
                    if PageLayout::update_document(&mut page, *slot, &new_doc).unwrap_or(false) {
                        *old_doc = new_doc;
                    }
                }
            }
            
            // Insert new documents
            for i in 0..5 {
                let size = 20 + (cycle * i) % 80;
                let doc = vec![b'Z'; size];
                if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
                    active_slots.push((slot, doc));
                }
            }
            
            // Verify all active documents are still retrievable and correct
            for (slot, expected_doc) in &active_slots {
                let retrieved = PageLayout::get_document(&page, *slot)
                    .expect("Failed to retrieve document");
                assert_eq!(retrieved, *expected_doc, "Document corruption detected");
            }
            
            // Verify page consistency
            let count = PageLayout::get_document_count(&page).expect("Failed to get count");
            assert_eq!(count as usize, active_slots.len(), "Document count mismatch");
        }
        
        // Final compaction and verification
        PageLayout::compact_page(&mut page).expect("Failed to compact");
        
        for (slot, expected_doc) in &active_slots {
            let retrieved = PageLayout::get_document(&page, *slot)
                .expect("Document lost after compaction");
            assert_eq!(retrieved, *expected_doc, "Document corrupted by compaction");
        }
    }

    /// Test boundary conditions around page capacity
    #[test]
    fn test_page_capacity_boundaries() {
        let mut page = create_test_page();
        let mut inserted_docs = Vec::new();
        
        // Fill page to near capacity with documents of varying sizes
        let mut total_inserted = 0;
        for size in [1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
            let doc = vec![b'A'; size];
            match PageLayout::insert_document(&mut page, &doc) {
                Ok(slot) => {
                    inserted_docs.push((slot, doc.clone()));
                    total_inserted += 1;
                    
                    // Verify immediate retrieval
                    let retrieved = PageLayout::get_document(&page, slot).expect("Failed to retrieve");
                    assert_eq!(retrieved, doc);
                }
                Err(_) => {
                    // Page is full - this is acceptable
                    break;
                }
            }
        }
        
        assert!(total_inserted > 0, "Should have inserted at least some documents");
        
        // Try to insert various sizes when page is near/at capacity
        for size in [1, 10, 100, 1000, 5000] {
            let doc = vec![b'B'; size];
            let result = PageLayout::insert_document(&mut page, &doc);
            // Should either succeed or fail gracefully
            if let Ok(slot) = result {
                let retrieved = PageLayout::get_document(&page, slot).expect("Failed to retrieve");
                assert_eq!(retrieved, doc);
                inserted_docs.push((slot, doc));
            }
        }
        
        // Verify all previously inserted documents are still intact
        for (slot, expected_doc) in &inserted_docs {
            let retrieved = PageLayout::get_document(&page, *slot)
                .expect("Document lost at capacity boundary");
            assert_eq!(retrieved, *expected_doc);
        }
        
        // Test utilization calculation at capacity boundary
        let utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
        assert!(utilization >= 0.0 && utilization <= 100.0);
    }

    /// Test slot directory behavior under extreme conditions
    #[test]
    fn test_slot_directory_extremes() {
        let mut page = create_test_page();
        let mut slots = Vec::new();
        
        // Create many small documents to stress slot directory
        for i in 0..100 {
            let doc = vec![b'A' + (i % 26) as u8; 1];
            match PageLayout::insert_document(&mut page, &doc) {
                Ok(slot) => slots.push(slot),
                Err(_) => break, // Page full
            }
        }
        
        if slots.is_empty() {
            return; // Can't test if no slots were created
        }
        
        // Delete every other slot to create fragmentation
        for i in (0..slots.len()).step_by(2) {
            PageLayout::delete_document(&mut page, slots[i]).expect("Failed to delete");
        }
        
        // Insert new documents - they should reuse deleted slots
        let mut reuse_count = 0;
        for _i in 0..slots.len() / 2 {
            let doc = vec![b'Z'; 2]; // Slightly larger documents
            if let Ok(new_slot) = PageLayout::insert_document(&mut page, &doc) {
                // Check if slot was reused
                if slots.iter().any(|&s| s == new_slot) {
                    reuse_count += 1;
                }
                
                // Verify document was stored correctly
                let retrieved = PageLayout::get_document(&page, new_slot).expect("Failed to retrieve");
                assert_eq!(retrieved, doc);
            }
        }
        
        // Should have reused some slots
        assert!(reuse_count > 0, "No slots were reused");
        
        // Verify remaining original documents are still accessible
        for i in (1..slots.len()).step_by(2) {
            let result = PageLayout::get_document(&page, slots[i]);
            assert!(result.is_ok(), "Original document should still be accessible");
        }
    }

    /// Test update operations under various constraint scenarios
    #[test]
    fn test_constrained_update_scenarios() {
        let mut page = create_test_page();
        
        // Fill page with documents of specific sizes
        let sizes = vec![100, 200, 150, 75, 300, 50];
        let mut slots = Vec::new();
        
        for size in sizes {
            let doc = vec![b'A'; size];
            if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
                slots.push((slot, size));
            }
        }
        
        if slots.is_empty() {
            return; // Can't test without documents
        }
        
        // Test updates that should fit in place
        for &(slot, original_size) in &slots {
            let new_size = original_size / 2; // Smaller size should always fit
            let new_doc = vec![b'B'; new_size];
            
            let result = PageLayout::update_document(&mut page, slot, &new_doc);
            assert!(result.is_ok() && result.unwrap(), "Smaller update should succeed");
            
            let retrieved = PageLayout::get_document(&page, slot).expect("Failed to retrieve updated doc");
            assert_eq!(retrieved, new_doc);
        }
        
        // Test updates that require relocation
        for &(slot, original_size) in &slots {
            let new_size = original_size * 3; // Much larger size
            let new_doc = vec![b'C'; new_size];
            
            let result = PageLayout::update_document(&mut page, slot, &new_doc);
            // May succeed or fail depending on available space
            if let Ok(true) = result {
                let retrieved = PageLayout::get_document(&page, slot).expect("Failed to retrieve updated doc");
                assert_eq!(retrieved, new_doc);
            }
        }
        
        // Verify page consistency after all updates
        let count = PageLayout::get_document_count(&page).expect("Failed to get count");
        assert!(count > 0, "Should still have documents after updates");
        
        let utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
        assert!(utilization >= 0.0 && utilization <= 100.0, "Invalid utilization after updates");
    }

    /// Test page state transitions and recovery
    #[test]
    fn test_page_state_transitions() {
        let mut page = create_test_page();
        
        // Test empty -> full -> fragmented -> compacted -> full cycle
        let mut phase_documents = Vec::new();
        
        // Phase 1: Fill page
        for i in 0..50 {
            let doc = vec![b'A' + (i % 26) as u8; 50 + (i % 50)];
            if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
                phase_documents.push((slot, doc));
            } else {
                break;
            }
        }
        
        let full_count = PageLayout::get_document_count(&page).expect("Failed to get count");
        let _full_utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
        
        // Phase 2: Create fragmentation
        let to_delete = phase_documents.len() / 3;
        for i in 0..to_delete {
            let idx = i * 3; // Delete every third document
            if idx < phase_documents.len() {
                let (slot, _) = &phase_documents[idx];
                PageLayout::delete_document(&mut page, *slot).expect("Failed to delete");
            }
        }
        
        let fragmented_count = PageLayout::get_document_count(&page).expect("Failed to get count");
        assert!(fragmented_count < full_count, "Count should decrease after deletions");
        
        // Phase 3: Compact
        PageLayout::compact_page(&mut page).expect("Failed to compact");
        
        let compacted_count = PageLayout::get_document_count(&page).expect("Failed to get count");
        assert_eq!(compacted_count, fragmented_count, "Count should remain same after compaction");
        
        let compacted_utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
        assert!(compacted_utilization >= 0.0 && compacted_utilization <= 100.0);
        
        // Phase 4: Verify remaining documents are intact
        for (i, (slot, expected_doc)) in phase_documents.iter().enumerate() {
            let result = PageLayout::get_document(&page, *slot);
            
            // Check if this document was actually deleted: idx = i must equal some j*3 where j < to_delete
            let was_actually_deleted = i % 3 == 0 && i < to_delete * 3;
            
            if was_actually_deleted {
                // This document was deleted
                assert!(result.is_err(), "Deleted document should not be retrievable");
            } else {
                // This document should still exist
                let retrieved = result.expect("Remaining document should be retrievable");
                assert_eq!(retrieved, *expected_doc, "Document corrupted during state transitions");
            }
        }
        
        // Phase 5: Fill remaining space
        for _i in 0..20 {
            let doc = vec![b'Z'; 30];
            let result = PageLayout::insert_document(&mut page, &doc);
            if let Ok(slot) = result {
                let retrieved = PageLayout::get_document(&page, slot).expect("Failed to retrieve new doc");
                assert_eq!(retrieved, doc);
            }
        }
        
        let final_utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
        assert!(final_utilization >= compacted_utilization, "Utilization should increase or stay same");
    }

    /// Test error handling and recovery scenarios
    #[test]
    fn test_error_recovery_scenarios() {
        let mut page = create_test_page();
        
        // Insert some valid documents first
        let mut valid_slots = Vec::new();
        for i in 0..10 {
            let doc = vec![b'A' + i as u8; 50];
            if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
                valid_slots.push((slot, doc));
            }
        }
        
        // Test sequence of operations that should fail gracefully
        
        // Try to insert empty document
        let result = PageLayout::insert_document(&mut page, &[]);
        assert!(result.is_err(), "Empty document should be rejected");
        
        // Try to delete invalid slot
        let result = PageLayout::delete_document(&mut page, 9999);
        assert!(result.is_err(), "Invalid slot deletion should fail");
        
        // Try to update invalid slot
        let result = PageLayout::update_document(&mut page, 9999, b"test");
        assert!(result.is_err(), "Invalid slot update should fail");
        
        // Try to get invalid slot
        let result = PageLayout::get_document(&page, 9999);
        assert!(result.is_err(), "Invalid slot retrieval should fail");
        
        // Verify that all original documents are still intact after error conditions
        for (slot, expected_doc) in &valid_slots {
            let retrieved = PageLayout::get_document(&page, *slot)
                .expect("Valid document should still be retrievable after errors");
            assert_eq!(retrieved, *expected_doc, "Document corrupted by error conditions");
        }
        
        // Verify page statistics are still consistent
        let count = PageLayout::get_document_count(&page).expect("Failed to get count");
        assert_eq!(count as usize, valid_slots.len(), "Document count inconsistent after errors");
        
        let utilization = PageLayout::get_utilization_percentage(&page).expect("Failed to get utilization");
        assert!(utilization >= 0.0 && utilization <= 100.0, "Invalid utilization after errors");
        
        // Test that page is still functional after error conditions
        let new_doc = vec![b'Z'; 30];
        if let Ok(slot) = PageLayout::insert_document(&mut page, &new_doc) {
            let retrieved = PageLayout::get_document(&page, slot).expect("Failed to retrieve after errors");
            assert_eq!(retrieved, new_doc);
        }
    }

    /// Test memory layout and bounds checking
    #[test]
    fn test_memory_layout_integrity() {
        let mut page = create_test_page();
        
        // Test with documents of sizes that could cause alignment issues
        let tricky_sizes = vec![1, 3, 5, 7, 9, 15, 17, 31, 33, 63, 65, 127, 129, 255, 257];
        let mut stored_docs = Vec::new();
        
        for size in tricky_sizes {
            let doc = vec![b'X'; size];
            if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
                stored_docs.push((slot, doc));
            }
        }
        
        // Verify all documents can be retrieved correctly
        for (slot, expected_doc) in &stored_docs {
            let retrieved = PageLayout::get_document(&page, *slot)
                .expect("Failed to retrieve document with tricky size");
            assert_eq!(retrieved, *expected_doc, "Document with tricky size was corrupted");
        }
        
        // Test alternating insert/delete pattern
        for i in 0..5 {
            // Delete half the documents
            let to_delete = stored_docs.len() / 2;
            for _ in 0..to_delete {
                if !stored_docs.is_empty() {
                    let (slot, _) = stored_docs.remove(0);
                    PageLayout::delete_document(&mut page, slot).expect("Failed to delete");
                }
            }
            
            // Insert new documents with different patterns
            for j in 0..3 {
                let size = 10 + (i * 10) + j;
                let doc = vec![b'Y'; size];
                if let Ok(slot) = PageLayout::insert_document(&mut page, &doc) {
                    stored_docs.push((slot, doc));
                }
            }
            
            // Verify all remaining documents are still correct
            for (slot, expected_doc) in &stored_docs {
                let retrieved = PageLayout::get_document(&page, *slot)
                    .expect("Document lost during alternating pattern");
                assert_eq!(retrieved, *expected_doc, "Document corrupted during alternating pattern");
            }
        }
        
        // Final compaction and verification
        PageLayout::compact_page(&mut page).expect("Failed to compact");
        
        for (slot, expected_doc) in &stored_docs {
            let retrieved = PageLayout::get_document(&page, *slot)
                .expect("Document lost after final compaction");
            assert_eq!(retrieved, *expected_doc, "Document corrupted by final compaction");
        }
    }
}
