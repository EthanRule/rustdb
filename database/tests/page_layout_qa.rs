use database::storage::{page::{Page, PageType}, page_layout::PageLayout};

fn create_test_page() -> Page {
    let mut page = Page::new(1, PageType::Data);
    PageLayout::initialize_page(&mut page).expect("Failed to initialize page");
    page
}

/// Generate test data of specific size
fn generate_test_data(size: usize, pattern: u8) -> Vec<u8> {
    vec![pattern; size]
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_maximum_slots_per_page() {
        let mut page = create_test_page();
        let mut inserted_slots = Vec::new();
        
        // Try to insert tiny documents until we hit slot limit
        let tiny_doc = b"x";
        
        // Insert until we can't anymore
        loop {
            match PageLayout::insert_document(&mut page, tiny_doc) {
                Ok(slot_id) => {
                    inserted_slots.push(slot_id);
                }
                Err(_) => break,
            }
            
            // Safety check to prevent infinite loop
            if inserted_slots.len() > 2000 {
                break;
            }
        }
        
        println!("Successfully inserted {} documents", inserted_slots.len());
        assert!(inserted_slots.len() > 100, "Should be able to insert many tiny documents");
        
        // Verify all documents are readable
        for slot_id in &inserted_slots {
            let data = PageLayout::get_document(&page, *slot_id).unwrap();
            assert_eq!(data, tiny_doc);
        }
        
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), inserted_slots.len() as u16);
    }

    #[test]
    fn test_fill_page_to_capacity() {
        let mut page = create_test_page();
        let mut total_inserted = 0;
        let mut document_sizes = Vec::new();
        
        // Insert documents of varying sizes until page is full
        for size in (50..500).step_by(50) {
            loop {
                let doc = generate_test_data(size, (size % 256) as u8);
                match PageLayout::insert_document(&mut page, &doc) {
                    Ok(_) => {
                        total_inserted += 1;
                        document_sizes.push(size);
                    }
                    Err(_) => break,
                }
                
                // Safety check
                if total_inserted > 1000 {
                    break;
                }
            }
        }
        
        println!("Inserted {} documents with sizes: {:?}", total_inserted, document_sizes);
        
        // Verify utilization is high
        let utilization = PageLayout::get_utilization_percentage(&page).unwrap();
        println!("Page utilization: {:.2}%", utilization);
        assert!(utilization > 50.0, "Page should be well utilized");
        
        // Verify all documents are still readable
        let mut slot_id = 0;
        for &size in &document_sizes {
            let expected_data = generate_test_data(size, (size % 256) as u8);
            let actual_data = PageLayout::get_document(&page, slot_id).unwrap();
            assert_eq!(actual_data, expected_data, "Document at slot {} corrupted", slot_id);
            slot_id += 1;
        }
    }

    #[test]
    fn test_massive_slot_reuse_cycle() {
        let mut page = create_test_page();
        
        // First, fill up some slots
        let mut slots = Vec::new();
        for i in 0..50 {
            let doc = format!("Document {}", i).into_bytes();
            let slot_id = PageLayout::insert_document(&mut page, &doc).unwrap();
            slots.push(slot_id);
        }
        
        // Now repeatedly delete and re-insert to test slot reuse
        for cycle in 0..100 {
            // Delete every other slot
            for &slot_id in slots.iter().step_by(2) {
                PageLayout::delete_document(&mut page, slot_id).unwrap();
            }
            
            // Re-insert documents (should reuse deleted slots)
            for _i in slots.iter().step_by(2) {
                let doc = format!("Cycle {} Replacement", cycle).into_bytes();
                let new_slot = PageLayout::insert_document(&mut page, &doc).unwrap();
                // The new slot should reuse one of the deleted slots
                assert!(slots.contains(&new_slot), "New slot should reuse deleted slot");
            }
            
            // Verify document count remains consistent
            assert_eq!(PageLayout::get_document_count(&page).unwrap(), 50);
        }
    }
}

#[cfg(test)]
mod edge_case_tests {
    use super::*;

    #[test]
    fn test_single_byte_documents() {
        let mut page = create_test_page();
        
        // Insert 256 different single-byte documents
        for byte_value in 0..=255u8 {
            let doc = vec![byte_value];
            let slot_id = PageLayout::insert_document(&mut page, &doc).unwrap();
            
            let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
            assert_eq!(retrieved, vec![byte_value]);
        }
        
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 256);
    }

    #[test]
    fn test_update_to_exact_same_data() {
        let mut page = create_test_page();
        let doc = b"Unchanged document";
        
        let slot_id = PageLayout::insert_document(&mut page, doc).unwrap();
        
        // Update with exactly the same data
        let success = PageLayout::update_document(&mut page, slot_id, doc).unwrap();
        assert!(success);
        
        let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
        assert_eq!(retrieved, doc);
    }

    #[test]
    fn test_alternating_sizes_update() {
        let mut page = create_test_page();
        let small_doc = b"Small";
        let large_doc = b"This is a much larger document with more content";
        
        let slot_id = PageLayout::insert_document(&mut page, small_doc).unwrap();
        
        // Alternate between small and large updates
        for i in 0..10 {
            if i % 2 == 0 {
                let success = PageLayout::update_document(&mut page, slot_id, large_doc).unwrap();
                assert!(success);
                assert_eq!(PageLayout::get_document(&page, slot_id).unwrap(), large_doc);
            } else {
                let success = PageLayout::update_document(&mut page, slot_id, small_doc).unwrap();
                assert!(success);
                assert_eq!(PageLayout::get_document(&page, slot_id).unwrap(), small_doc);
            }
        }
    }

    #[test]
    fn test_complex_deletion_pattern() {
        let mut page = create_test_page();
        let mut slots = Vec::new();
        
        // Insert 20 documents
        for i in 0..20 {
            let doc = format!("Document number {}", i).into_bytes();
            let slot_id = PageLayout::insert_document(&mut page, &doc).unwrap();
            slots.push(slot_id);
        }
        
        // Delete in a complex pattern: prime indices
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19];
        for &prime in &primes {
            if prime < slots.len() {
                PageLayout::delete_document(&mut page, slots[prime]).unwrap();
            }
        }
        
        let expected_remaining = 20 - primes.len();
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), expected_remaining as u16);
        
        // Verify non-deleted documents are still accessible
        for (i, &slot_id) in slots.iter().enumerate() {
            if !primes.contains(&i) {
                let expected_doc = format!("Document number {}", i).into_bytes();
                let actual_doc = PageLayout::get_document(&page, slot_id).unwrap();
                assert_eq!(actual_doc, expected_doc);
            }
        }
    }

    #[test]
    fn test_zero_byte_boundary_document() {
        let mut page = create_test_page();
        
        // Test documents that are powers of 2 minus 1 (boundary conditions)
        let sizes = vec![1, 3, 7, 15, 31, 63, 127, 255, 511, 1023];
        
        for size in sizes {
            let doc = generate_test_data(size, 0xAB);
            let slot_id = PageLayout::insert_document(&mut page, &doc).unwrap();
            let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
            assert_eq!(retrieved.len(), size);
            assert_eq!(retrieved, doc);
        }
    }

    #[test]
    fn test_slot_directory_growth_boundary() {
        let mut page = create_test_page();
        let doc = b"test";
        
        // Insert documents and verify slot directory grows correctly
        for expected_slot in 0..100 {
            let actual_slot = PageLayout::insert_document(&mut page, doc).unwrap();
            assert_eq!(actual_slot, expected_slot, "Slot assignment should be sequential");
            
            // Verify document count
            assert_eq!(PageLayout::get_document_count(&page).unwrap(), expected_slot + 1);
        }
        
        // Verify all documents are still accessible
        for slot_id in 0..100 {
            let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
            assert_eq!(retrieved, doc);
        }
    }
}

#[cfg(test)]
mod data_integrity_tests {
    use super::*;

    #[test]
    fn test_document_isolation() {
        let mut page = create_test_page();
        
        // Insert documents with different patterns
        let patterns = vec![
            vec![0x00; 100],
            vec![0xFF; 100], 
            vec![0xAA; 100],
            vec![0x55; 100],
            (0..100).collect::<Vec<u8>>(),
        ];
        
        let mut slots = Vec::new();
        for (i, pattern) in patterns.iter().enumerate() {
            let slot_id = PageLayout::insert_document(&mut page, pattern).unwrap();
            slots.push((slot_id, i));
        }
        
        // Verify each document maintains its pattern
        for (slot_id, pattern_index) in slots {
            let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
            assert_eq!(retrieved, patterns[pattern_index], "Document pattern corrupted");
        }
    }

    #[test]
    fn test_fragmentation_and_compaction_integrity() {
        let mut page = create_test_page();
        
        // Create a fragmented page
        let mut slots = Vec::new();
        for i in 0..10 {
            let doc = format!("Document {}", i).repeat(10).into_bytes();
            let slot_id = PageLayout::insert_document(&mut page, &doc).unwrap();
            slots.push((slot_id, doc));
        }
        
        // Delete every other document to create fragmentation
        for i in (0..slots.len()).step_by(2) {
            PageLayout::delete_document(&mut page, slots[i].0).unwrap();
        }
        
        // Compact the page
        PageLayout::compact_page(&mut page).unwrap();
        
        // Verify remaining documents are intact
        for i in (1..slots.len()).step_by(2) {
            let (slot_id, expected_doc) = &slots[i];
            let actual_doc = PageLayout::get_document(&page, *slot_id).unwrap();
            assert_eq!(actual_doc, *expected_doc, "Document corrupted after compaction");
        }
    }

    #[test] 
    fn test_large_document_integrity() {
        let mut page = create_test_page();
        
        // Create a large document with a verifiable pattern
        let large_size = 2048;
        let mut large_doc = Vec::with_capacity(large_size);
        for i in 0..large_size {
            large_doc.push((i % 256) as u8);
        }
        
        let slot_id = PageLayout::insert_document(&mut page, &large_doc).unwrap();
        let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
        
        assert_eq!(retrieved.len(), large_size);
        for i in 0..large_size {
            assert_eq!(retrieved[i], (i % 256) as u8, "Byte {} corrupted", i);
        }
    }

    #[test]
    fn test_update_preserves_other_documents() {
        let mut page = create_test_page();
        
        // Insert multiple documents
        let docs = vec![
            b"Document A".to_vec(),
            b"Document B with more content".to_vec(), 
            b"Document C".to_vec(),
        ];
        
        let mut slots = Vec::new();
        for doc in &docs {
            let slot_id = PageLayout::insert_document(&mut page, doc).unwrap();
            slots.push(slot_id);
        }
        
        // Update the middle document
        let new_middle_doc = b"Updated middle document with different size";
        PageLayout::update_document(&mut page, slots[1], new_middle_doc).unwrap();
        
        // Verify other documents are unchanged
        assert_eq!(PageLayout::get_document(&page, slots[0]).unwrap(), docs[0]);
        assert_eq!(PageLayout::get_document(&page, slots[2]).unwrap(), docs[2]);
        assert_eq!(PageLayout::get_document(&page, slots[1]).unwrap(), new_middle_doc);
    }
}

#[cfg(test)]
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_double_delete_error() {
        let mut page = create_test_page();
        let doc = b"Document to delete";
        
        let slot_id = PageLayout::insert_document(&mut page, doc).unwrap();
        
        // First delete should succeed
        assert!(PageLayout::delete_document(&mut page, slot_id).is_ok());
        
        // Second delete should fail
        assert!(PageLayout::delete_document(&mut page, slot_id).is_err());
    }

    #[test]
    fn test_invalid_slot_operations() {
        let mut page = create_test_page();
        
        // Test operations on non-existent slots
        assert!(PageLayout::get_document(&page, 0).is_err());
        assert!(PageLayout::delete_document(&mut page, 0).is_err());
        assert!(PageLayout::update_document(&mut page, 0, b"data").is_err());
        
        // Insert one document
        let slot_id = PageLayout::insert_document(&mut page, b"test").unwrap();
        
        // Test operations on out-of-range slots
        assert!(PageLayout::get_document(&page, slot_id + 1).is_err());
        assert!(PageLayout::delete_document(&mut page, slot_id + 1).is_err());
        assert!(PageLayout::update_document(&mut page, slot_id + 1, b"data").is_err());
    }

    #[test]
    fn test_operations_on_deleted_slots() {
        let mut page = create_test_page();
        let doc = b"Document to delete";
        
        let slot_id = PageLayout::insert_document(&mut page, doc).unwrap();
        PageLayout::delete_document(&mut page, slot_id).unwrap();
        
        // All operations on deleted slot should fail
        assert!(PageLayout::get_document(&page, slot_id).is_err());
        assert!(PageLayout::update_document(&mut page, slot_id, b"new data").is_err());
    }

    #[test]
    fn test_empty_document_rejection() {
        let mut page = create_test_page();
        
        // Empty document should be rejected
        assert!(PageLayout::insert_document(&mut page, &[]).is_err());
    }
}

#[cfg(test)]
mod consistency_tests {
    use super::*;

    #[test]
    fn test_document_count_consistency() {
        let mut page = create_test_page();
        let doc = b"Test document";
        
        // Initially empty
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 0);
        
        // Insert documents and verify count
        for i in 1..=10 {
            PageLayout::insert_document(&mut page, doc).unwrap();
            assert_eq!(PageLayout::get_document_count(&page).unwrap(), i);
        }
        
        // Delete documents and verify count  
        for i in (0..10).step_by(2) {
            PageLayout::delete_document(&mut page, i).unwrap();
        }
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 5);
        
        // Compact and verify count unchanged
        PageLayout::compact_page(&mut page).unwrap();
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 5);
    }

    #[test]
    fn test_utilization_calculation_consistency() {
        let mut page = create_test_page();
        
        // Empty page
        assert_eq!(PageLayout::get_utilization_percentage(&page).unwrap(), 0.0);
        
        // Add documents and verify utilization increases
        let mut last_utilization = 0.0;
        for i in 0..5 {
            let doc = format!("Document {}", i).repeat(20).into_bytes();
            PageLayout::insert_document(&mut page, &doc).unwrap();
            
            let current_utilization = PageLayout::get_utilization_percentage(&page).unwrap();
            assert!(current_utilization > last_utilization, "Utilization should increase");
            assert!(current_utilization <= 100.0, "Utilization should not exceed 100%");
            last_utilization = current_utilization;
        }
        
        // Delete a document and verify utilization decreases
        PageLayout::delete_document(&mut page, 0).unwrap();
        let utilization_after_delete = PageLayout::get_utilization_percentage(&page).unwrap();
        assert!(utilization_after_delete < last_utilization, "Utilization should decrease after deletion");
    }

    #[test]
    fn test_slot_assignment_consistency() {
        let mut page = create_test_page();
        let doc = b"Test document";
        
        // Insert documents and verify sequential slot assignment
        for expected_slot in 0..10 {
            let actual_slot = PageLayout::insert_document(&mut page, doc).unwrap();
            assert_eq!(actual_slot, expected_slot);
        }
        
        // Delete some slots
        PageLayout::delete_document(&mut page, 2).unwrap();
        PageLayout::delete_document(&mut page, 5).unwrap();
        PageLayout::delete_document(&mut page, 8).unwrap();
        
        // Next insertions should reuse deleted slots
        let reused_slots = vec![
            PageLayout::insert_document(&mut page, doc).unwrap(),
            PageLayout::insert_document(&mut page, doc).unwrap(), 
            PageLayout::insert_document(&mut page, doc).unwrap(),
        ];
        
        // Should reuse the deleted slots (in some order)
        let expected_reused = vec![2, 5, 8];
        for slot in reused_slots {
            assert!(expected_reused.contains(&slot), "Should reuse deleted slot");
        }
    }
}

#[cfg(test)]
mod performance_tests {
    use super::*;

    #[test]
    fn test_insertion_performance_pattern() {
        let mut page = create_test_page();
        let doc = b"Performance test document";
        
        let start = std::time::Instant::now();
        
        // Insert many documents
        let mut count = 0;
        loop {
            match PageLayout::insert_document(&mut page, doc) {
                Ok(_) => count += 1,
                Err(_) => break,
            }
            
            // Safety limit
            if count > 1000 {
                break;
            }
        }
        
        let duration = start.elapsed();
        println!("Inserted {} documents in {:?}", count, duration);
        
        // Performance should be reasonable (this is just a sanity check)
        assert!(count > 50, "Should be able to insert a reasonable number of documents");
        assert!(duration.as_millis() < 1000, "Insertion should be reasonably fast");
    }

    #[test]
    fn test_retrieval_performance() {
        let mut page = create_test_page();
        let mut slots = Vec::new();
        
        // Insert documents
        for i in 0..100 {
            let doc = format!("Document {}", i).into_bytes();
            match PageLayout::insert_document(&mut page, &doc) {
                Ok(slot_id) => slots.push(slot_id),
                Err(_) => break,
            }
        }
        
        let start = std::time::Instant::now();
        
        // Retrieve all documents multiple times
        for _ in 0..10 {
            for &slot_id in &slots {
                let _doc = PageLayout::get_document(&page, slot_id).unwrap();
            }
        }
        
        let duration = start.elapsed();
        let total_retrievals = slots.len() * 10;
        
        println!("Retrieved {} documents in {:?}", total_retrievals, duration);
        assert!(duration.as_millis() < 1000, "Retrieval should be reasonably fast");
    }
}
