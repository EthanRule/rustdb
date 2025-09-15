use crate::error::DatabaseError;
use crate::storage::file::DatabaseFile;
use crate::storage::page::Page;
use std::collections::HashMap;

pub struct BufferPool {
    // Maximum number of pages in buffer pool
    capacity: usize,
    // Current pages in memory
    pages: HashMap<u64, Page>, // page_id -> Page
    // LRU tracking: most recent at front, least recent at back
    lru_list: LruList,
    // Quick lookup for LRU nodes
    page_to_node: HashMap<u64, LruNodeId>,
    // Dirty pages that need to be written back
    dirty_pages: std::collections::HashSet<u64>,
    // Pinned pages (cannot be evicted)
    pinned_pages: std::collections::HashSet<u64>,
}

type LruNodeId = usize;

// Doubly linked list node for LRU tracking
#[derive(Debug)]
struct LruNode {
    page_id: u64,
    prev: Option<LruNodeId>,
    next: Option<LruNodeId>,
}

#[derive(Debug)]
struct LruList {
    nodes: Vec<LruNode>,
    head: Option<LruNodeId>,
    tail: Option<LruNodeId>,
    free_nodes: Vec<LruNodeId>, // Reuse freed node slots
}

impl BufferPool {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            pages: HashMap::new(),
            lru_list: LruList::new(),
            page_to_node: HashMap::new(),
            dirty_pages: std::collections::HashSet::new(),
            pinned_pages: std::collections::HashSet::new(),
        }
    }

    /// Pin a page in memory (prevents eviction)
    pub fn pin_page(
        &mut self,
        page_id: u64,
        database_file: &mut DatabaseFile,
    ) -> Result<&mut Page, DatabaseError> {
        // Check if page is already in buffer pool
        if let Some(_page) = self.pages.get(&page_id) {
            self.pinned_pages.insert(page_id);
            self.move_to_front(page_id);
            return Ok(self.pages.get_mut(&page_id).unwrap());
        }

        // If buffer pool is full, evict a page
        if self.pages.len() >= self.capacity {
            self.evict_page(database_file)?;
        }

        // Load page from disk (you'll need to implement this)
        let page = database_file.read_page(page_id)?;

        // Add to buffer pool
        self.pages.insert(page_id, page);
        self.pinned_pages.insert(page_id);
        self.add_to_front(page_id);

        Ok(self.pages.get_mut(&page_id).unwrap())
    }

    /// Unpin a page (allows eviction)
    pub fn unpin_page(&mut self, page_id: u64, is_dirty: bool) {
        self.pinned_pages.remove(&page_id);
        if is_dirty {
            self.dirty_pages.insert(page_id);
        }
    }

    /// Get read-only access to a page
    pub fn get_page(
        &mut self,
        page_id: u64,
        database_file: &mut DatabaseFile,
    ) -> Result<&Page, DatabaseError> {
        if self.pages.contains_key(&page_id) {
            self.move_to_front(page_id);
            return Ok(self.pages.get(&page_id).unwrap());
        }

        // Load from disk if not in buffer pool
        if self.pages.len() >= self.capacity {
            self.evict_page(database_file)?;
        }

        let page = self.load_page_from_disk(page_id, database_file)?;
        self.pages.insert(page_id, page);
        self.add_to_front(page_id);

        Ok(self.pages.get(&page_id).unwrap())
    }

    /// Evict least recently used page
    fn evict_page(&mut self, database_file: &mut DatabaseFile) -> Result<(), DatabaseError> {
        // Find LRU page that's not pinned
        let mut current = self.lru_list.tail;
        while let Some(node_id) = current {
            let node = &self.lru_list.nodes[node_id];
            let page_id = node.page_id;

            // Can't evict pinned pages
            if !self.pinned_pages.contains(&page_id) {
                // Write back if dirty
                if self.dirty_pages.contains(&page_id) {
                    self.write_page_to_disk(page_id, database_file)?;
                    self.dirty_pages.remove(&page_id);
                }

                // Remove from buffer pool
                self.pages.remove(&page_id);
                self.remove_from_lru(page_id);
                return Ok(());
            }

            current = node.prev;
        }

        Err(DatabaseError::Storage(
            "No pages available for eviction".to_string(),
        ))
    }

    /// Move page to front of LRU list (most recently used)
    fn move_to_front(&mut self, page_id: u64) {
        if let Some(&node_id) = self.page_to_node.get(&page_id) {
            self.lru_list.move_to_front(node_id);
        }
    }

    /// Add new page to front of LRU list
    fn add_to_front(&mut self, page_id: u64) {
        let node_id = self.lru_list.add_to_front(page_id);
        self.page_to_node.insert(page_id, node_id);
    }

    /// Remove page from LRU list
    fn remove_from_lru(&mut self, page_id: u64) {
        if let Some(node_id) = self.page_to_node.remove(&page_id) {
            self.lru_list.remove(node_id);
        }
    }

    fn load_page_from_disk(
        &self,
        page_id: u64,
        database_file: &mut DatabaseFile,
    ) -> Result<Page, DatabaseError> {
        let page = database_file.read_page(page_id)?;

        if page.get_page_id() != page_id {
            return Err(DatabaseError::Storage(format!(
                "Page ID mismatch! Expected {}. got {}",
                page_id,
                page.get_page_id()
            )));
        }

        Ok(page)
    }

    fn write_page_to_disk(
        &self,
        page_id: u64,
        database_file: &mut DatabaseFile,
    ) -> Result<(), DatabaseError> {
        if let Some(page) = self.pages.get(&page_id) {
            database_file.write_page(page_id, page)?;
        } else {
            return Err(DatabaseError::Storage(format!(
                "Page {} was not found in buffer pool",
                page_id
            )));
        }
        Ok(())
    }

    /// Get buffer pool statistics
    pub fn get_stats(&self) -> BufferPoolStats {
        BufferPoolStats {
            capacity: self.capacity,
            pages_in_pool: self.pages.len(),
            dirty_pages: self.dirty_pages.len(),
            pinned_pages: self.pinned_pages.len(),
        }
    }

    /// Resize the buffer pool capacity
    pub fn resize(
        &mut self,
        new_capacity: usize,
        database_file: &mut DatabaseFile,
    ) -> Result<(), DatabaseError> {
        if new_capacity == 0 {
            return Err(DatabaseError::Storage(
                "Buffer pool capacity cannot be zero".to_string(),
            ));
        }

        let old_capacity = self.capacity;
        self.capacity = new_capacity;

        // If shrinking, we need to evict pages
        while self.pages.len() > new_capacity {
            self.evict_page(database_file)?;
        }

        // Log the resize operation
        #[cfg(debug_assertions)]
        eprintln!(
            "Buffer pool resized from {} to {} pages",
            old_capacity, new_capacity
        );

        Ok(())
    }

    /// Force flush all dirty pages to disk
    pub fn flush_all(&mut self, database_file: &mut DatabaseFile) -> Result<(), DatabaseError> {
        let dirty_page_ids: Vec<u64> = self.dirty_pages.iter().cloned().collect();

        for page_id in dirty_page_ids {
            self.write_page_to_disk(page_id, database_file)?;
            self.dirty_pages.remove(&page_id);
        }

        Ok(())
    }

    /// Force flush a specific page to disk
    pub fn flush_page(
        &mut self,
        page_id: u64,
        database_file: &mut DatabaseFile,
    ) -> Result<(), DatabaseError> {
        if self.dirty_pages.contains(&page_id) {
            self.write_page_to_disk(page_id, database_file)?;
            self.dirty_pages.remove(&page_id);
        }
        Ok(())
    }

    /// Clear all pages from buffer pool (for testing/debugging)
    pub fn clear(&mut self, database_file: &mut DatabaseFile) -> Result<(), DatabaseError> {
        // Flush all dirty pages first
        self.flush_all(database_file)?;

        // Clear all data structures
        self.pages.clear();
        self.dirty_pages.clear();
        self.pinned_pages.clear();
        self.page_to_node.clear();
        self.lru_list = LruList::new();

        Ok(())
    }

    /// Get detailed buffer pool statistics
    pub fn get_detailed_stats(&self) -> DetailedBufferPoolStats {
        let lru_chain = self.get_lru_chain();

        DetailedBufferPoolStats {
            capacity: self.capacity,
            pages_in_pool: self.pages.len(),
            dirty_pages: self.dirty_pages.len(),
            pinned_pages: self.pinned_pages.len(),
            utilization_percentage: (self.pages.len() as f64 / self.capacity as f64) * 100.0,
            lru_chain_length: lru_chain.len(),
            free_nodes_count: self.lru_list.free_nodes.len(),
            pages_in_lru: lru_chain,
        }
    }

    /// Get the LRU chain for debugging
    fn get_lru_chain(&self) -> Vec<u64> {
        let mut chain = Vec::new();
        let mut current = self.lru_list.head;

        while let Some(node_id) = current {
            let node = &self.lru_list.nodes[node_id];
            chain.push(node.page_id);
            current = node.next;
        }

        chain
    }

    /// Debug print buffer pool state
    pub fn debug_print(&self) {
        println!("=== Buffer Pool Debug Info ===");
        println!("Capacity: {}", self.capacity);
        println!("Pages in pool: {}", self.pages.len());
        println!("Dirty pages: {:?}", self.dirty_pages);
        println!("Pinned pages: {:?}", self.pinned_pages);
        println!("LRU chain (head to tail): {:?}", self.get_lru_chain());
        println!("Free nodes: {:?}", self.lru_list.free_nodes);
        println!("Page to node mapping: {:?}", self.page_to_node);
        println!("===============================");
    }

    /// Check if a page is in the buffer pool
    pub fn contains_page(&self, page_id: u64) -> bool {
        self.pages.contains_key(&page_id)
    }

    /// Check if a page is dirty
    pub fn is_dirty(&self, page_id: u64) -> bool {
        self.dirty_pages.contains(&page_id)
    }

    /// Check if a page is pinned
    pub fn is_pinned(&self, page_id: u64) -> bool {
        self.pinned_pages.contains(&page_id)
    }

    /// Get all page IDs currently in the buffer pool
    pub fn get_all_page_ids(&self) -> Vec<u64> {
        self.pages.keys().cloned().collect()
    }

    /// Force evict a specific page (for testing)
    pub fn force_evict_page(
        &mut self,
        page_id: u64,
        database_file: &mut DatabaseFile,
    ) -> Result<(), DatabaseError> {
        if self.pinned_pages.contains(&page_id) {
            return Err(DatabaseError::Storage(
                "Cannot evict pinned page".to_string(),
            ));
        }

        if self.dirty_pages.contains(&page_id) {
            self.write_page_to_disk(page_id, database_file)?;
            self.dirty_pages.remove(&page_id);
        }

        self.pages.remove(&page_id);
        self.remove_from_lru(page_id);

        Ok(())
    }

    /// Validate buffer pool internal consistency (for testing)
    pub fn validate_consistency(&self) -> Result<(), String> {
        // Check that all pages in the buffer pool are in the LRU list
        let lru_pages: std::collections::HashSet<u64> = self.get_lru_chain().into_iter().collect();
        let buffer_pages: std::collections::HashSet<u64> = self.pages.keys().cloned().collect();

        if lru_pages != buffer_pages {
            return Err(format!(
                "LRU chain and buffer pool pages don't match. LRU: {:?}, Buffer: {:?}",
                lru_pages, buffer_pages
            ));
        }

        // Check that page_to_node mapping is consistent
        for (page_id, &node_id) in &self.page_to_node {
            if node_id >= self.lru_list.nodes.len() {
                return Err(format!("Invalid node_id {} for page {}", node_id, page_id));
            }

            if self.lru_list.nodes[node_id].page_id != *page_id {
                return Err(format!(
                    "Node {} has page_id {} but should have {}",
                    node_id, self.lru_list.nodes[node_id].page_id, page_id
                ));
            }
        }

        // Check that dirty and pinned pages are in the buffer pool
        for &page_id in &self.dirty_pages {
            if !self.pages.contains_key(&page_id) {
                return Err(format!("Dirty page {} not in buffer pool", page_id));
            }
        }

        for &page_id in &self.pinned_pages {
            if !self.pages.contains_key(&page_id) {
                return Err(format!("Pinned page {} not in buffer pool", page_id));
            }
        }

        Ok(())
    }
}

impl LruList {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
            free_nodes: Vec::new(),
        }
    }

    fn add_to_front(&mut self, page_id: u64) -> LruNodeId {
        let node_id = if let Some(free_id) = self.free_nodes.pop() {
            // Reuse a freed node slot
            self.nodes[free_id] = LruNode {
                page_id,
                prev: None,
                next: self.head,
            };
            free_id
        } else {
            // Create new node
            let node_id = self.nodes.len();
            self.nodes.push(LruNode {
                page_id,
                prev: None,
                next: self.head,
            });
            node_id
        };

        // Update head pointer
        if let Some(old_head) = self.head {
            self.nodes[old_head].prev = Some(node_id);
        }
        self.head = Some(node_id);

        // Update tail if this is the first node
        if self.tail.is_none() {
            self.tail = Some(node_id);
        }

        node_id
    }

    fn move_to_front(&mut self, node_id: LruNodeId) {
        // If already at front, nothing to do
        if self.head == Some(node_id) {
            return;
        }

        // Remove from current position
        let node = &self.nodes[node_id];
        let prev = node.prev;
        let next = node.next;

        if let Some(prev_id) = prev {
            self.nodes[prev_id].next = next;
        }
        if let Some(next_id) = next {
            self.nodes[next_id].prev = prev;
        }

        // Update tail if we're removing the tail
        if self.tail == Some(node_id) {
            self.tail = prev;
        }

        // Move to front
        self.nodes[node_id].prev = None;
        self.nodes[node_id].next = self.head;

        if let Some(old_head) = self.head {
            self.nodes[old_head].prev = Some(node_id);
        }
        self.head = Some(node_id);
    }

    fn remove(&mut self, node_id: LruNodeId) {
        let node = &self.nodes[node_id];
        let prev = node.prev;
        let next = node.next;

        if let Some(prev_id) = prev {
            self.nodes[prev_id].next = next;
        } else {
            // Removing head
            self.head = next;
        }

        if let Some(next_id) = next {
            self.nodes[next_id].prev = prev;
        } else {
            // Removing tail
            self.tail = prev;
        }

        // Mark node as free for reuse
        self.free_nodes.push(node_id);
    }
}

#[derive(Debug)]
pub struct BufferPoolStats {
    pub capacity: usize,
    pub pages_in_pool: usize,
    pub dirty_pages: usize,
    pub pinned_pages: usize,
}

#[derive(Debug)]
pub struct DetailedBufferPoolStats {
    pub capacity: usize,
    pub pages_in_pool: usize,
    pub dirty_pages: usize,
    pub pinned_pages: usize,
    pub utilization_percentage: f64,
    pub lru_chain_length: usize,
    pub free_nodes_count: usize,
    pub pages_in_lru: Vec<u64>,
}

#[cfg(test)]
mod tests {
    // use super::*;
}
