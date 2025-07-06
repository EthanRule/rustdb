use database::storage::buffer_pool::BufferPool;

#[test]
fn test_buffer_pool_integration() {
    // Create a small buffer pool for testing
    let pool = BufferPool::new(3);
    
    // Test initial state
    let stats = pool.get_stats();
    assert_eq!(stats.capacity, 3);
    assert_eq!(stats.pages_in_pool, 0);
    assert_eq!(stats.dirty_pages, 0);
    assert_eq!(stats.pinned_pages, 0);
    
    // Test detailed stats
    let detailed_stats = pool.get_detailed_stats();
    assert_eq!(detailed_stats.utilization_percentage, 0.0);
    assert_eq!(detailed_stats.lru_chain_length, 0);
    
    // Test basic operations
    assert!(!pool.contains_page(1));
    assert!(!pool.is_dirty(1));
    assert!(!pool.is_pinned(1));
    
    // Test page tracking
    assert_eq!(pool.get_all_page_ids().len(), 0);
    
    // Test consistency validation
    assert!(pool.validate_consistency().is_ok());
    
    // Test debug print (just ensure it doesn't panic)
    pool.debug_print();
}

#[test]
fn test_buffer_pool_resize_and_management() {
    let mut pool = BufferPool::new(5);
    
    // Test resizing
    assert!(pool.resize(10).is_ok());
    assert_eq!(pool.get_stats().capacity, 10);
    
    // Test shrinking
    assert!(pool.resize(3).is_ok());
    assert_eq!(pool.get_stats().capacity, 3);
    
    // Test invalid resize
    assert!(pool.resize(0).is_err());
    
    // Test clear
    assert!(pool.clear().is_ok());
    
    let stats = pool.get_stats();
    assert_eq!(stats.pages_in_pool, 0);
    assert_eq!(stats.dirty_pages, 0);
    assert_eq!(stats.pinned_pages, 0);
}

#[test]
fn test_buffer_pool_lru_simulation() {
    // Test LRU behavior without actual disk I/O
    let pool = BufferPool::new(3);
    
    // Since we can't add pages without actual disk operations,
    // just test that the LRU chain is empty initially
    let lru_chain = pool.get_detailed_stats().pages_in_lru;
    assert_eq!(lru_chain, Vec::<u64>::new());
    
    // Test consistency
    assert!(pool.validate_consistency().is_ok());
}

#[test]
fn test_buffer_pool_stats_and_diagnostics() {
    let mut pool = BufferPool::new(100);
    
    // Test various stats methods
    let basic_stats = pool.get_stats();
    let detailed_stats = pool.get_detailed_stats();
    
    // Stats should be consistent
    assert_eq!(basic_stats.capacity, detailed_stats.capacity);
    assert_eq!(basic_stats.pages_in_pool, detailed_stats.pages_in_pool);
    assert_eq!(basic_stats.dirty_pages, detailed_stats.dirty_pages);
    assert_eq!(basic_stats.pinned_pages, detailed_stats.pinned_pages);
    
    // Test utilization calculation
    assert_eq!(detailed_stats.utilization_percentage, 0.0);
    
    // Test unpin operations (these should not affect consistency since pages aren't in pool)
    pool.unpin_page(1, false);  // Don't mark as dirty
    pool.unpin_page(2, false);  // Don't mark as dirty
    
    // Test force eviction (should succeed on empty pool)
    assert!(pool.force_evict_page(999).is_ok());
    
    // Test that these operations maintain consistency
    assert!(pool.validate_consistency().is_ok());
}
