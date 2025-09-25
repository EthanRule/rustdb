use database::storage::{page::{Page, PageType}, page_layout::PageLayout};
use std::collections::{HashMap, HashSet};

fn create_test_page() -> Page {
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    page
}

/// Generate deterministic but varied test data
fn generate_test_document(seed: u64, size: usize) -> Vec<u8> {
    let mut doc = Vec::with_capacity(size);
    let mut rng_state = seed;
    
    for _ in 0..size {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        doc.push((rng_state >> 16) as u8);
    }
    
    doc
}

/// Property-based tests using deterministic pseudo-random data
#[cfg(test)]
mod property_tests {
    use super::*;

    #[test]
    fn property_insert_then_retrieve_always_equal() {
        let mut page = create_test_page();
        let mut inserted_docs = HashMap::new();
        
        // Test with various document sizes and patterns
        for seed in 0..100 {
            let size = (seed % 500) + 1; // Size between 1 and 500
            let doc = generate_test_document(seed, size as usize);
            
            if let Ok(slot_id) = PageLayout::insert_document(&mut page, &doc) {
                inserted_docs.insert(slot_id, doc.clone());
                
                // Property: inserted document should always be retrievable and equal
                let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
                assert_eq!(retrieved, doc, "Insert-retrieve property violated for seed {}", seed);
            }
        }
        
        // Verify all inserted documents are still correct
        for (slot_id, expected_doc) in inserted_docs {
            let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
            assert_eq!(retrieved, expected_doc, "Document consistency violated for slot {}", slot_id);
        }
    }

    #[test]
    fn property_document_count_reflects_operations() {
        let mut page = create_test_page();
        let mut expected_count = 0u16;
        let mut active_slots = HashSet::new();
        
        // Perform a sequence of operations
        for op_seed in 0..200 {
            let operation = op_seed % 3;
            
            match operation {
                0 => {
                    // Insert
                    let doc_size = (op_seed % 100) + 1;
                    let doc = generate_test_document(op_seed, doc_size as usize);
                    
                    if let Ok(slot_id) = PageLayout::insert_document(&mut page, &doc) {
                        if active_slots.insert(slot_id) {
                            expected_count += 1;
                        }
                    }
                }
                1 => {
                    // Delete (if we have documents)
                    if !active_slots.is_empty() {
                        let slot_to_delete = *active_slots.iter().nth(op_seed as usize % active_slots.len()).unwrap();
                        if PageLayout::delete_document(&mut page, slot_to_delete).is_ok() {
                            active_slots.remove(&slot_to_delete);
                            expected_count -= 1;
                        }
                    }
                }
                2 => {
                    // Update (if we have documents)
                    if !active_slots.is_empty() {
                        let slot_to_update = *active_slots.iter().nth(op_seed as usize % active_slots.len()).unwrap();
                        let new_doc_size = (op_seed % 100) + 1;
                        let new_doc = generate_test_document(op_seed + 1000, new_doc_size as usize);
                        
                        // Update doesn't change count
                        let _ = PageLayout::update_document(&mut page, slot_to_update, &new_doc);
                    }
                }
                _ => unreachable!(),
            }
            
            // Property: document count should always match our tracking
            let actual_count = PageLayout::get_document_count(&page).unwrap();
            assert_eq!(actual_count, expected_count, 
                      "Document count property violated after operation {} (seed {})", operation, op_seed);
        }
    }

    #[test]
    fn property_slot_reuse_is_deterministic() {
        let mut page = create_test_page();
        let doc = b"Test document";
        
        // Insert documents to fill some slots
        let mut slots = Vec::new();
        for _ in 0..10 {
            if let Ok(slot_id) = PageLayout::insert_document(&mut page, doc) {
                slots.push(slot_id);
            }
        }
        
        // Delete specific slots
        let deleted_slots = vec![1, 3, 7];
        for &slot in &deleted_slots {
            PageLayout::delete_document(&mut page, slot).unwrap();
        }
        
        // Insert new documents - should reuse deleted slots
        let mut reused_slots = Vec::new();
        for _ in 0..3 {
            if let Ok(slot_id) = PageLayout::insert_document(&mut page, doc) {
                reused_slots.push(slot_id);
            }
        }
        
        // Property: reused slots should be from the deleted set
        for &reused_slot in &reused_slots {
            assert!(deleted_slots.contains(&reused_slot), 
                   "Slot reuse property violated: {} not in deleted slots", reused_slot);
        }
    }

    #[test]
    fn property_update_preserves_other_documents() {
        let mut page = create_test_page();
        let mut documents = HashMap::new();
        
        // Insert multiple documents
        for i in 0..20 {
            let doc = generate_test_document(i, ((i % 50) + 1) as usize);
            if let Ok(slot_id) = PageLayout::insert_document(&mut page, &doc) {
                documents.insert(slot_id, doc);
            }
        }
        
        let slots: Vec<_> = documents.keys().cloned().collect();
        
        // Update each document and verify others are unchanged
        for &update_slot in &slots {
            let new_doc = generate_test_document(999 + update_slot as u64, 50);
            
            // Take snapshot of other documents before update
            let mut other_docs = HashMap::new();
            for (&slot, doc) in &documents {
                if slot != update_slot {
                    other_docs.insert(slot, doc.clone());
                }
            }
            
            // Perform update
            if PageLayout::update_document(&mut page, update_slot, &new_doc).unwrap() {
                documents.insert(update_slot, new_doc.clone());
                
                // Property: other documents should be unchanged
                for (slot, expected_doc) in other_docs {
                    let actual_doc = PageLayout::get_document(&page, slot).unwrap();
                    assert_eq!(actual_doc, expected_doc, 
                              "Update isolation property violated: slot {} changed when updating slot {}", 
                              slot, update_slot);
                }
            }
        }
    }

    #[test]
    fn property_compaction_preserves_active_documents() {
        let mut page = create_test_page();
        let mut original_documents = HashMap::new();
        
        // Insert documents
        for i in 0..30 {
            let doc = generate_test_document(i, ((i % 100) + 10) as usize);
            if let Ok(slot_id) = PageLayout::insert_document(&mut page, &doc) {
                original_documents.insert(slot_id, doc);
            }
        }
        
        // Delete some documents
        let mut deleted_slots = HashSet::new();
        let mut remaining_documents = HashMap::new();
        
        for (&slot, doc) in &original_documents {
            if slot % 3 == 0 {
                PageLayout::delete_document(&mut page, slot).unwrap();
                deleted_slots.insert(slot);
            } else {
                remaining_documents.insert(slot, doc.clone());
            }
        }
        
        // Compact the page
        PageLayout::compact_page(&mut page).unwrap();
        
        // Property: all remaining documents should be accessible and unchanged
        for (slot, expected_doc) in remaining_documents {
            let actual_doc = PageLayout::get_document(&page, slot).unwrap();
            assert_eq!(actual_doc, expected_doc, 
                      "Compaction preservation property violated for slot {}", slot);
        }
        
        // Property: deleted documents should not be accessible
        for deleted_slot in deleted_slots {
            assert!(PageLayout::get_document(&page, deleted_slot).is_err(),
                   "Deleted document still accessible after compaction: slot {}", deleted_slot);
        }
    }

    #[test]
    fn property_utilization_percentage_bounds() {
        let mut page = create_test_page();
        
        // Property: utilization should always be between 0 and 100
        assert_eq!(PageLayout::get_utilization_percentage(&page).unwrap(), 0.0);
        
        let mut operation_count = 0;
        for seed in 0..100 {
            let operation = seed % 4;
            
            match operation {
                0 => {
                    // Insert
                    let doc_size = (seed % 200) + 1;
                    let doc = generate_test_document(seed, doc_size as usize);
                    let _ = PageLayout::insert_document(&mut page, &doc);
                }
                1 => {
                    // Delete random slot
                    if operation_count > 5 {
                        let slot = (seed % 10) as u16;
                        let _ = PageLayout::delete_document(&mut page, slot);
                    }
                }
                2 => {
                    // Update random slot
                    if operation_count > 5 {
                        let slot = (seed % 10) as u16;
                        let new_doc = generate_test_document(seed + 500, 50);
                        let _ = PageLayout::update_document(&mut page, slot, &new_doc);
                    }
                }
                3 => {
                    // Compact
                    if operation_count > 10 && operation_count % 20 == 0 {
                        let _ = PageLayout::compact_page(&mut page);
                    }
                }
                _ => unreachable!(),
            }
            
            operation_count += 1;
            
            // Property: utilization must be in valid range
            let utilization = PageLayout::get_utilization_percentage(&page).unwrap();
            assert!(utilization >= 0.0 && utilization <= 100.0, 
                   "Utilization bounds property violated: {}% after operation {}", utilization, operation);
        }
    }
}

/// Fuzzing-style tests with pseudo-random operations
#[cfg(test)]
mod fuzz_tests {
    use super::*;

    #[test]
    fn fuzz_random_operations_sequence() {
        let mut page = create_test_page();
        let mut slot_tracker = HashSet::new();
        let mut operation_count = 0;
        
        // Perform many random operations
        for seed in 0..1000 {
            let operation = seed % 6;
            
            match operation {
                0..=2 => {
                    // Insert (higher probability)
                    let doc_size = (seed % 300) + 1;
                    let doc = generate_test_document(seed, doc_size as usize);
                    
                    if let Ok(slot_id) = PageLayout::insert_document(&mut page, &doc) {
                        slot_tracker.insert(slot_id);
                        
                        // Verify immediate retrieval
                        let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
                        assert_eq!(retrieved, doc, "Immediate retrieval failed for seed {}", seed);
                    }
                }
                3 => {
                    // Delete
                    if !slot_tracker.is_empty() {
                        let slot_vec: Vec<_> = slot_tracker.iter().cloned().collect();
                        let slot_to_delete = slot_vec[seed as usize % slot_vec.len()];
                        
                        if PageLayout::delete_document(&mut page, slot_to_delete).is_ok() {
                            slot_tracker.remove(&slot_to_delete);
                        }
                    }
                }
                4 => {
                    // Update
                    if !slot_tracker.is_empty() {
                        let slot_vec: Vec<_> = slot_tracker.iter().cloned().collect();
                        let slot_to_update = slot_vec[seed as usize % slot_vec.len()];
                        let new_doc_size = (seed % 200) + 1;
                        let new_doc = generate_test_document(seed + 10000, new_doc_size as usize);
                        
                        let _ = PageLayout::update_document(&mut page, slot_to_update, &new_doc);
                    }
                }
                5 => {
                    // Compact (occasionally)
                    if operation_count > 50 && operation_count % 100 == 0 {
                        PageLayout::compact_page(&mut page).unwrap();
                    }
                }
                _ => unreachable!(),
            }
            
            operation_count += 1;
            
            // Invariant checks
            let doc_count = PageLayout::get_document_count(&page).unwrap();
            assert!(doc_count <= slot_tracker.len() as u16, 
                   "Document count invariant violated: count={}, tracked={}", doc_count, slot_tracker.len());
            
            let utilization = PageLayout::get_utilization_percentage(&page).unwrap();
            assert!(utilization >= 0.0 && utilization <= 100.0, 
                   "Utilization invariant violated: {}", utilization);
            
            // Periodically verify all tracked documents are still accessible
            if operation_count % 50 == 0 {
                for &slot in &slot_tracker {
                    assert!(PageLayout::get_document(&page, slot).is_ok(), 
                           "Tracked document became inaccessible: slot {}", slot);
                }
            }
        }
        
        println!("Completed {} operations successfully", operation_count);
        println!("Final state: {} tracked slots, {}% utilization", 
                slot_tracker.len(), PageLayout::get_utilization_percentage(&page).unwrap());
    }

    #[test]
    fn fuzz_extreme_document_sizes() {
        let _page = create_test_page();
        
        // Test with extreme but valid document sizes
        let test_sizes = vec![
            1, 2, 3, 4, 5, 7, 8, 15, 16, 31, 32, 63, 64, 127, 128, 255, 256,
            511, 512, 1023, 1024, 2047, 2048, 4095, 4096, 8191
        ];
        
        for (i, &size) in test_sizes.iter().enumerate() {
            let mut new_page = create_test_page();
            let doc = generate_test_document(i as u64, size);
            
            if let Ok(slot_id) = PageLayout::insert_document(&mut new_page, &doc) {
                let retrieved = PageLayout::get_document(&new_page, slot_id).unwrap();
                assert_eq!(retrieved.len(), size, "Size mismatch for document of size {}", size);
                assert_eq!(retrieved, doc, "Content mismatch for document of size {}", size);
                
                // Test update with different size
                let new_size = if size > 100 { size - 50 } else { size + 50 };
                let new_doc = generate_test_document((i + 1000) as u64, new_size);
                
                if PageLayout::update_document(&mut new_page, slot_id, &new_doc).unwrap() {
                    let updated_retrieved = PageLayout::get_document(&new_page, slot_id).unwrap();
                    assert_eq!(updated_retrieved, new_doc, "Update failed for size {} -> {}", size, new_size);
                }
            }
        }
    }

    #[test]
    fn fuzz_concurrent_slot_operations() {
        let mut page = create_test_page();
        
        // Simulate interleaved operations on different slots
        let mut slots = Vec::new();
        
        // Create initial slots
        for i in 0..20 {
            let doc = generate_test_document(i, 50);
            if let Ok(slot_id) = PageLayout::insert_document(&mut page, &doc) {
                slots.push((slot_id, doc));
            }
        }
        
        // Perform interleaved operations
        for round in 0..50 {
            let operation = round % 4;
            let slot_index = round % slots.len();
            
            match operation {
                0 => {
                    // Update slot
                    let (slot_id, _) = &slots[slot_index];
                    let new_doc = generate_test_document((round + 1000) as u64, ((round % 100) + 1) as usize);
                    
                    if PageLayout::update_document(&mut page, *slot_id, &new_doc).unwrap() {
                        slots[slot_index].1 = new_doc;
                    }
                }
                1 => {
                    // Delete and re-insert
                    let (slot_id, _) = slots[slot_index].clone();
                    
                    if PageLayout::delete_document(&mut page, slot_id).is_ok() {
                        let new_doc = generate_test_document((round + 2000) as u64, 60);
                        if let Ok(new_slot_id) = PageLayout::insert_document(&mut page, &new_doc) {
                            slots[slot_index] = (new_slot_id, new_doc);
                        }
                    }
                }
                2 => {
                    // Read all slots to verify consistency
                    for (slot_id, expected_doc) in &slots {
                        if let Ok(actual_doc) = PageLayout::get_document(&page, *slot_id) {
                            assert_eq!(actual_doc, *expected_doc, 
                                      "Consistency check failed for slot {} in round {}", slot_id, round);
                        }
                    }
                }
                3 => {
                    // Occasional compaction
                    if round % 15 == 0 {
                        PageLayout::compact_page(&mut page).unwrap();
                    }
                }
                _ => unreachable!(),
            }
        }
        
        // Final consistency check
        for (slot_id, expected_doc) in &slots {
            if let Ok(actual_doc) = PageLayout::get_document(&page, *slot_id) {
                assert_eq!(actual_doc, *expected_doc, "Final consistency check failed for slot {}", slot_id);
            }
        }
    }
}
