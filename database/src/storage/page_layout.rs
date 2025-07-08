use crate::storage::page::{Page, PAGE_SIZE};
use crate::error::DatabaseError;
use std::mem;

pub type SlotId = u16;

// Page layout constants
const SLOT_DIRECTORY_OFFSET: usize = PAGE_SIZE - 4; // Last 4 bytes for slot directory header
const MAX_SLOTS_PER_PAGE: u16 = 1000;
const SLOT_SIZE: usize = 4; // Each slot is 4 bytes (offset: u16, length: u16)
const TOMBSTONE_MARKER: u16 = 0xFFFF;

/// Slot directory header stored at the end of the page
#[repr(C)]
#[derive(Debug)]
struct SlotDirectoryHeader {
    slot_count: u16,
    free_space_offset: u16, // Pointer to start of free space
}

/// Individual slot entry (offset and length of document)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct SlotEntry {
    offset: u16, // Offset from start of page to document data
    length: u16, // Length of document data (0xFFFF for tombstone)
}

impl SlotEntry {
    fn new(offset: u16, length: u16) -> Self {
        Self { offset, length }
    }
    
    fn tombstone() -> Self {
        Self { offset: 0, length: TOMBSTONE_MARKER }
    }
    
    fn is_tombstone(&self) -> bool {
        self.length == TOMBSTONE_MARKER
    }
    
    fn is_empty(&self) -> bool {
        self.offset == 0 && self.length == 0
    }
}

/// Page layout manager for document storage with slot directory
pub struct PageLayout;

impl PageLayout {
    /// Initialize a new page for document storage
    pub fn initialize_page(page: &mut Page) -> Result<(), DatabaseError> {
        let header = SlotDirectoryHeader {
            slot_count: 0,
            free_space_offset: Self::get_header_size() as u16,
        };
        
        Self::write_slot_directory_header(page, &header)?;
        Self::update_page_free_space(page)?;
        Ok(())
    }
    
    /// Insert a document into the page and return its slot ID
    pub fn insert_document(page: &mut Page, document_bytes: &[u8]) -> Result<SlotId, DatabaseError> {
        if document_bytes.is_empty() {
            return Err(DatabaseError::Storage("Cannot insert empty document".to_string()));
        }
        
        let doc_size = document_bytes.len();
        if doc_size > u16::MAX as usize {
            return Err(DatabaseError::Storage("Document too large".to_string()));
        }
        
        let header = Self::read_slot_directory_header(page)?;
        
        // Find an empty or tombstoned slot
        let (slot_id, is_new_slot) = if let Some(slot_id) = Self::find_reusable_slot(page, &header)? {
            (slot_id, false)
        } else {
            // Need a new slot
            if header.slot_count >= MAX_SLOTS_PER_PAGE {
                return Err(DatabaseError::Storage("Maximum slots per page exceeded".to_string()));
            }
            (header.slot_count, true)
        };
        
        // Calculate the slot count we'll have after this insertion
        let final_slot_count = if is_new_slot {
            header.slot_count + 1
        } else {
            header.slot_count
        };
        
        // Check if we have enough space (including space for new slot if needed)
        let required_space = doc_size + if is_new_slot { SLOT_SIZE } else { 0 };
        if !Self::has_sufficient_space_with_count(page, required_space, final_slot_count)? {
            return Err(DatabaseError::Storage("Insufficient space on page".to_string()));
        }
        
        // Find space for the document
        let doc_offset = Self::find_free_space_with_count(page, doc_size, final_slot_count)?;
        
        // Write the document data
        Self::write_document_data(page, doc_offset, document_bytes)?;
        
        // Update slot entry
        let slot_entry = SlotEntry::new(doc_offset, doc_size as u16);
        
        // Update header if we added a new slot
        if is_new_slot {
            // When adding a new slot, we need to shift all existing slots down
            // because the slot directory grows downward - do this BEFORE writing the new slot
            Self::shift_slot_directory_for_new_slot(page, header.slot_count)?;
            
            let new_header = SlotDirectoryHeader {
                slot_count: final_slot_count,
                free_space_offset: header.free_space_offset,
            };
            Self::write_slot_directory_header(page, &new_header)?;
        }
        
        // Write slot entry using the final slot count for correct offset calculation
        Self::write_slot_entry_with_count(page, slot_id, &slot_entry, final_slot_count)?;
        
        // Update page free space
        Self::update_page_free_space(page)?;
        
        Ok(slot_id)
    }
    
    /// Get a document by its slot ID - returns owned data
    pub fn get_document(page: &Page, slot_id: SlotId) -> Result<Vec<u8>, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        
        if slot_id >= header.slot_count {
            return Err(DatabaseError::Storage("Invalid slot ID".to_string()));
        }
        
        let slot_entry = Self::read_slot_entry(page, slot_id)?;
        
        if slot_entry.is_tombstone() {
            return Err(DatabaseError::Storage("Document has been deleted".to_string()));
        }
        
        if slot_entry.is_empty() {
            return Err(DatabaseError::Storage("Empty slot".to_string()));
        }
        
        Self::read_document_data_owned(page, slot_entry.offset, slot_entry.length)
    }
    
    /// Delete a document by marking it with a tombstone
    pub fn delete_document(page: &mut Page, slot_id: SlotId) -> Result<(), DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        
        if slot_id >= header.slot_count {
            return Err(DatabaseError::Storage("Invalid slot ID".to_string()));
        }
        
        let slot_entry = Self::read_slot_entry(page, slot_id)?;
        
        if slot_entry.is_tombstone() {
            return Err(DatabaseError::Storage("Document already deleted".to_string()));
        }
        
        if slot_entry.is_empty() {
            return Err(DatabaseError::Storage("Empty slot".to_string()));
        }
        
        // Mark slot as tombstone
        let tombstone_entry = SlotEntry::tombstone();
        Self::write_slot_entry_with_count(page, slot_id, &tombstone_entry, header.slot_count)?;
        
        // Update page free space
        Self::update_page_free_space(page)?;
        
        Ok(())
    }
    
    /// Update a document in place, returns false if new data doesn't fit
    pub fn update_document(page: &mut Page, slot_id: SlotId, new_data: &[u8]) -> Result<bool, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        
        if slot_id >= header.slot_count {
            return Err(DatabaseError::Storage("Invalid slot ID".to_string()));
        }
        
        let slot_entry = Self::read_slot_entry(page, slot_id)?;
        
        if slot_entry.is_tombstone() {
            return Err(DatabaseError::Storage("Document has been deleted".to_string()));
        }
        
        if slot_entry.is_empty() {
            return Err(DatabaseError::Storage("Empty slot".to_string()));
        }
        
        let new_size = new_data.len();
        if new_size > u16::MAX as usize {
            return Err(DatabaseError::Storage("Document too large".to_string()));
        }
        
        // If new data fits in existing space, update in place
        if new_size <= slot_entry.length as usize {
            Self::write_document_data(page, slot_entry.offset, new_data)?;
            
            // Update slot entry with new length
            let updated_entry = SlotEntry::new(slot_entry.offset, new_size as u16);
            Self::write_slot_entry(page, slot_id, &updated_entry)?;
            
            Self::update_page_free_space(page)?;
            return Ok(true);
        }
        
        // Check if we have space for the larger document
        let space_freed = slot_entry.length as usize;
        let space_needed = new_size;
        let net_space_needed = if space_needed > space_freed {
            space_needed - space_freed
        } else {
            0
        };
        
        if !Self::has_sufficient_space(page, net_space_needed)? {
            return Ok(false); // Doesn't fit
        }
        
        // Find new space for the document
        let new_offset = Self::find_free_space(page, new_size)?;
        
        // Write new document data
        Self::write_document_data(page, new_offset, new_data)?;
        
        // Update slot entry
        let updated_entry = SlotEntry::new(new_offset, new_size as u16);
        Self::write_slot_entry(page, slot_id, &updated_entry)?;
        
        Self::update_page_free_space(page)?;
        Ok(true)
    }
    
    /// Compact the page by removing fragmentation
    pub fn compact_page(page: &mut Page) -> Result<(), DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        
        // Collect all active documents
        let mut documents = Vec::new();
        
        for slot_id in 0..header.slot_count {
            let slot_entry = Self::read_slot_entry(page, slot_id)?;
            
            if !slot_entry.is_tombstone() && !slot_entry.is_empty() {
                let doc_data = Self::read_document_data_owned(page, slot_entry.offset, slot_entry.length)?;
                documents.push((slot_id, doc_data));
            }
        }
        
        // Clear all slot entries
        for slot_id in 0..header.slot_count {
            let empty_entry = SlotEntry { offset: 0, length: 0 };
            Self::write_slot_entry(page, slot_id, &empty_entry)?;
        }
        
        // Rewrite documents starting from the beginning of data area
        let mut current_offset = Self::get_header_size() as u16;
        
        for (slot_id, doc_data) in documents {
            Self::write_document_data(page, current_offset, &doc_data)?;
            
            let slot_entry = SlotEntry::new(current_offset, doc_data.len() as u16);
            Self::write_slot_entry(page, slot_id, &slot_entry)?;
            
            current_offset += doc_data.len() as u16;
        }
        
        // Update free space offset
        let new_header = SlotDirectoryHeader {
            slot_count: header.slot_count,
            free_space_offset: current_offset,
        };
        Self::write_slot_directory_header(page, &new_header)?;
        
        Self::update_page_free_space(page)?;
        Ok(())
    }
    
    /// Get page utilization percentage
    pub fn get_utilization_percentage(page: &Page) -> Result<f32, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        let usable_space = Self::get_usable_page_size(header.slot_count);
        let used_space = Self::get_used_space(page)?;
        
        if usable_space == 0 {
            return Ok(0.0);
        }
        
        Ok((used_space as f32 / usable_space as f32) * 100.0)
    }
    
    /// Get the number of documents stored in the page
    pub fn get_document_count(page: &Page) -> Result<u16, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        let mut count = 0;
        
        for slot_id in 0..header.slot_count {
            let slot_entry = Self::read_slot_entry(page, slot_id)?;
            if !slot_entry.is_tombstone() && !slot_entry.is_empty() {
                count += 1;
            }
        }
        
        Ok(count)
    }
    
    // Helper methods
    
    fn get_header_size() -> usize {
        // PageHeader is private, so we'll calculate it from page constants
        // Based on page.rs: page_id (u64) + page_type (u8) + free_space (u16) + checksum (u32) = 15 bytes
        // But with alignment, it's likely 16 bytes
        16 // This should match mem::size_of::<PageHeader>()
    }
    
    fn get_usable_page_size(slot_count: u16) -> usize {
        PAGE_SIZE - Self::get_header_size() - mem::size_of::<SlotDirectoryHeader>() - (slot_count as usize * SLOT_SIZE)
    }
    
    fn has_sufficient_space(page: &Page, required_space: usize) -> Result<bool, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        let usable_space = Self::get_usable_page_size(header.slot_count);
        let used_space = Self::get_used_space(page)?;
        
        Ok(usable_space >= used_space + required_space)
    }
    
    fn has_sufficient_space_with_count(page: &Page, required_size: usize, slot_count: u16) -> Result<bool, DatabaseError> {
        let used_space = Self::get_used_space(page)?;
        let usable_space = Self::get_usable_page_size(slot_count);
        Ok(used_space + required_size <= usable_space)
    }
    
    fn get_used_space(page: &Page) -> Result<usize, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        let mut used_space = 0;
        
        for slot_id in 0..header.slot_count {
            let slot_entry = Self::read_slot_entry(page, slot_id)?;
            if !slot_entry.is_tombstone() && !slot_entry.is_empty() {
                used_space += slot_entry.length as usize;
            }
        }
        
        Ok(used_space)
    }
    
    fn find_reusable_slot(page: &Page, header: &SlotDirectoryHeader) -> Result<Option<SlotId>, DatabaseError> {
        for slot_id in 0..header.slot_count {
            let slot_entry = Self::read_slot_entry(page, slot_id)?;
            if slot_entry.is_tombstone() || slot_entry.is_empty() {
                return Ok(Some(slot_id));
            }
        }
        Ok(None)
    }
    
    fn find_free_space(page: &Page, size: usize) -> Result<u16, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        
        // Simple strategy: allocate from the end of used space
        // In a more sophisticated implementation, this would find holes created by deletions
        let mut max_offset = Self::get_header_size() as u16;
        
        for slot_id in 0..header.slot_count {
            let slot_entry = Self::read_slot_entry(page, slot_id)?;
            if !slot_entry.is_tombstone() && !slot_entry.is_empty() {
                let end_offset = slot_entry.offset + slot_entry.length;
                if end_offset > max_offset {
                    max_offset = end_offset;
                }
            }
        }
        
        let available_space = Self::get_slot_directory_start(header.slot_count) - max_offset as usize;
        if available_space < size {
            return Err(DatabaseError::Storage("Insufficient contiguous space".to_string()));
        }
        
        Ok(max_offset)
    }
    
    fn find_free_space_with_count(page: &Page, size: usize, slot_count: u16) -> Result<u16, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        
        // Simple strategy: allocate from the end of used space
        // In a more sophisticated implementation, this would find holes created by deletions
        let mut max_offset = Self::get_header_size() as u16;
        
        for slot_id in 0..header.slot_count {
            let slot_entry = Self::read_slot_entry(page, slot_id)?;
            if !slot_entry.is_tombstone() && !slot_entry.is_empty() {
                let end_offset = slot_entry.offset + slot_entry.length;
                if end_offset > max_offset {
                    max_offset = end_offset;
                }
            }
        }
        
        let available_space = Self::get_slot_directory_start(slot_count) - max_offset as usize;
        if available_space < size {
            return Err(DatabaseError::Storage("Insufficient contiguous space".to_string()));
        }
        
        Ok(max_offset)
    }
    
    fn get_slot_directory_start(slot_count: u16) -> usize {
        SLOT_DIRECTORY_OFFSET - (slot_count as usize * SLOT_SIZE)
    }
    
    fn update_page_free_space(page: &mut Page) -> Result<(), DatabaseError> {
        let used_space = Self::get_used_space(page)?;
        let header = Self::read_slot_directory_header(page)?;
        let usable_space = Self::get_usable_page_size(header.slot_count);
        let free_space = usable_space.saturating_sub(used_space) as u16;
        
        page.update_free_space(free_space);
        Ok(())
    }
    
    /// Shift the slot directory entries to make room for a new slot
    fn shift_slot_directory_for_new_slot(page: &mut Page, old_slot_count: u16) -> Result<(), DatabaseError> {
        if old_slot_count == 0 {
            return Ok(()); // No existing slots to move
        }
        
        let data = Self::get_page_data_mut(page);
        
        // Calculate old and new positions
        let old_dir_start = Self::get_slot_directory_start(old_slot_count);
        let new_dir_start = Self::get_slot_directory_start(old_slot_count + 1);
        
        // The new directory starts earlier (lower offset) than the old one
        // We need to move all slots toward the beginning of the page
        // Copy in forward order since we're moving to lower addresses
        for slot_id in 0..old_slot_count {
            let old_offset = old_dir_start + (slot_id as usize * SLOT_SIZE);
            let new_offset = new_dir_start + (slot_id as usize * SLOT_SIZE);
            
            // Read the slot entry from the old location
            let slot_bytes = [
                data[old_offset],
                data[old_offset + 1], 
                data[old_offset + 2],
                data[old_offset + 3],
            ];
            
            // Write to the new location
            data[new_offset..new_offset + SLOT_SIZE].copy_from_slice(&slot_bytes);
        }
        
        Ok(())
    }
    
    // Low-level data access methods
    
    fn read_slot_directory_header(page: &Page) -> Result<SlotDirectoryHeader, DatabaseError> {
        let data = page.to_bytes();
        let header_bytes = &data[SLOT_DIRECTORY_OFFSET..SLOT_DIRECTORY_OFFSET + mem::size_of::<SlotDirectoryHeader>()];
        
        if header_bytes.len() < mem::size_of::<SlotDirectoryHeader>() {
            return Err(DatabaseError::Storage("Invalid slot directory header".to_string()));
        }
        
        let slot_count = u16::from_le_bytes([header_bytes[0], header_bytes[1]]);
        let free_space_offset = u16::from_le_bytes([header_bytes[2], header_bytes[3]]);
        
        Ok(SlotDirectoryHeader { slot_count, free_space_offset })
    }
    
    fn write_slot_directory_header(page: &mut Page, header: &SlotDirectoryHeader) -> Result<(), DatabaseError> {
        let data = Self::get_page_data_mut(page);
        let header_bytes = &mut data[SLOT_DIRECTORY_OFFSET..SLOT_DIRECTORY_OFFSET + mem::size_of::<SlotDirectoryHeader>()];
        
        header_bytes[0..2].copy_from_slice(&header.slot_count.to_le_bytes());
        header_bytes[2..4].copy_from_slice(&header.free_space_offset.to_le_bytes());
        
        Ok(())
    }
    
    fn read_slot_entry(page: &Page, slot_id: SlotId) -> Result<SlotEntry, DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        let slot_offset = Self::get_slot_directory_start(header.slot_count) + (slot_id as usize * SLOT_SIZE);
        
        let data = page.to_bytes();
        if slot_offset + SLOT_SIZE > data.len() {
            return Err(DatabaseError::Storage("Invalid slot offset".to_string()));
        }
        
        let slot_bytes = &data[slot_offset..slot_offset + SLOT_SIZE];
        let offset = u16::from_le_bytes([slot_bytes[0], slot_bytes[1]]);
        let length = u16::from_le_bytes([slot_bytes[2], slot_bytes[3]]);
        
        Ok(SlotEntry { offset, length })
    }
    
    fn write_slot_entry(page: &mut Page, slot_id: SlotId, entry: &SlotEntry) -> Result<(), DatabaseError> {
        let header = Self::read_slot_directory_header(page)?;
        let slot_offset = Self::get_slot_directory_start(header.slot_count) + (slot_id as usize * SLOT_SIZE);
        
        let data = Self::get_page_data_mut(page);
        if slot_offset + SLOT_SIZE > data.len() {
            return Err(DatabaseError::Storage("Invalid slot offset".to_string()));
        }
        
        let slot_bytes = &mut data[slot_offset..slot_offset + SLOT_SIZE];
        slot_bytes[0..2].copy_from_slice(&entry.offset.to_le_bytes());
        slot_bytes[2..4].copy_from_slice(&entry.length.to_le_bytes());
        
        Ok(())
    }
    
    fn write_slot_entry_with_count(page: &mut Page, slot_id: SlotId, entry: &SlotEntry, slot_count: u16) -> Result<(), DatabaseError> {
        let slot_offset = Self::get_slot_directory_start(slot_count) + (slot_id as usize * SLOT_SIZE);
        
        let data = Self::get_page_data_mut(page);
        if slot_offset + SLOT_SIZE > data.len() {
            return Err(DatabaseError::Storage("Invalid slot offset".to_string()));
        }
        
        let slot_bytes = &mut data[slot_offset..slot_offset + SLOT_SIZE];
        slot_bytes[0..2].copy_from_slice(&entry.offset.to_le_bytes());
        slot_bytes[2..4].copy_from_slice(&entry.length.to_le_bytes());
        
        Ok(())
    }
    
    fn read_document_data_owned(page: &Page, offset: u16, length: u16) -> Result<Vec<u8>, DatabaseError> {
        let data = page.to_bytes();
        let start = offset as usize;
        let end = start + length as usize;
        
        if end > data.len() {
            return Err(DatabaseError::Storage("Invalid document bounds".to_string()));
        }
        
        Ok(data[start..end].to_vec())
    }
    
    fn write_document_data(page: &mut Page, offset: u16, data: &[u8]) -> Result<(), DatabaseError> {
        let page_data = Self::get_page_data_mut(page);
        let start = offset as usize;
        let end = start + data.len();
        
        if end > page_data.len() {
            return Err(DatabaseError::Storage("Document exceeds page bounds".to_string()));
        }
        
        page_data[start..end].copy_from_slice(data);
        Ok(())
    }
    
    // Helper method to get mutable access to page data
    fn get_page_data_mut(page: &mut Page) -> &mut [u8; PAGE_SIZE] {
        // This is unsafe but necessary for direct page manipulation
        unsafe {
            &mut *(page as *mut Page as *mut [u8; PAGE_SIZE])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::page::PageType;

    fn create_test_page() -> Page {
        let mut page = Page::new(1, PageType::Data);
        // Zero out the slot directory area to ensure clean state
        let data = unsafe { &mut *((&mut page) as *mut Page as *mut [u8; PAGE_SIZE]) };
        
        // Initialize slot directory header
        let header_offset = SLOT_DIRECTORY_OFFSET;
        data[header_offset..header_offset + 4].copy_from_slice(&[0, 0, 16, 0]); // slot_count=0, free_space_offset=16
        
        page
    }

    #[test]
    fn test_page_initialization() {
        let page = create_test_page();
        let header = PageLayout::read_slot_directory_header(&page).unwrap();
        
        assert_eq!(header.slot_count, 0);
        assert_eq!(header.free_space_offset, PageLayout::get_header_size() as u16);
    }

    #[test]
    fn test_insert_and_get_document() {
        let mut page = create_test_page();
        let doc_data = b"Hello, World!";
        
        let slot_id = PageLayout::insert_document(&mut page, doc_data).unwrap();
        assert_eq!(slot_id, 0);
        
        let retrieved_data = PageLayout::get_document(&page, slot_id).unwrap();
        assert_eq!(retrieved_data, doc_data);
    }

    #[test]
    fn test_insert_multiple_documents() {
        let mut page = create_test_page();
        
        let doc1 = b"Document 1";
        let doc2 = b"Document 2 with more content";
        let doc3 = b"Doc 3";
        
        let slot1 = PageLayout::insert_document(&mut page, doc1).unwrap();
        let slot2 = PageLayout::insert_document(&mut page, doc2).unwrap();
        let slot3 = PageLayout::insert_document(&mut page, doc3).unwrap();
        
        assert_eq!(slot1, 0);
        assert_eq!(slot2, 1);
        assert_eq!(slot3, 2);
        
        assert_eq!(PageLayout::get_document(&page, slot1).unwrap(), doc1);
        assert_eq!(PageLayout::get_document(&page, slot2).unwrap(), doc2);
        assert_eq!(PageLayout::get_document(&page, slot3).unwrap(), doc3);
        
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 3);
    }

    #[test]
    fn test_delete_document() {
        let mut page = create_test_page();
        let doc_data = b"Document to delete";
        
        let slot_id = PageLayout::insert_document(&mut page, doc_data).unwrap();
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 1);
        
        PageLayout::delete_document(&mut page, slot_id).unwrap();
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 0);
        
        // Should fail to get deleted document
        assert!(PageLayout::get_document(&page, slot_id).is_err());
        
        // Should fail to delete again
        assert!(PageLayout::delete_document(&mut page, slot_id).is_err());
    }

    #[test]
    fn test_update_document_same_size() {
        let mut page = create_test_page();
        let original_data = b"Original data";
        let updated_data = b"Updated  data"; // Same length
        
        let slot_id = PageLayout::insert_document(&mut page, original_data).unwrap();
        
        let success = PageLayout::update_document(&mut page, slot_id, updated_data).unwrap();
        assert!(success);
        
        let retrieved_data = PageLayout::get_document(&page, slot_id).unwrap();
        assert_eq!(retrieved_data, updated_data);
    }

    #[test]
    fn test_update_document_smaller_size() {
        let mut page = create_test_page();
        let original_data = b"This is a long document";
        let updated_data = b"Short doc";
        
        let slot_id = PageLayout::insert_document(&mut page, original_data).unwrap();
        
        let success = PageLayout::update_document(&mut page, slot_id, updated_data).unwrap();
        assert!(success);
        
        let retrieved_data = PageLayout::get_document(&page, slot_id).unwrap();
        assert_eq!(retrieved_data, updated_data);
    }

    #[test]
    fn test_update_document_larger_size() {
        let mut page = create_test_page();
        let original_data = b"Short";
        let updated_data = b"This is a much longer document that requires more space";
        
        let slot_id = PageLayout::insert_document(&mut page, original_data).unwrap();
        
        let success = PageLayout::update_document(&mut page, slot_id, updated_data).unwrap();
        assert!(success);
        
        let retrieved_data = PageLayout::get_document(&page, slot_id).unwrap();
        assert_eq!(retrieved_data, updated_data);
    }

    #[test]
    fn test_slot_reuse_after_delete() {
        let mut page = create_test_page();
        
        let doc1 = b"Document 1";
        let doc2 = b"Document 2";
        let doc3 = b"Document 3";
        
        let slot1 = PageLayout::insert_document(&mut page, doc1).unwrap();
        let slot2 = PageLayout::insert_document(&mut page, doc2).unwrap();
        let slot3 = PageLayout::insert_document(&mut page, doc3).unwrap();
        
        // Delete middle document
        PageLayout::delete_document(&mut page, slot2).unwrap();
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 2);
        
        // Insert new document - should reuse slot2
        let doc4 = b"Document 4";
        let slot4 = PageLayout::insert_document(&mut page, doc4).unwrap();
        assert_eq!(slot4, slot2); // Should reuse the deleted slot
        
        assert_eq!(PageLayout::get_document(&page, slot1).unwrap(), doc1);
        assert_eq!(PageLayout::get_document(&page, slot4).unwrap(), doc4);
        assert_eq!(PageLayout::get_document(&page, slot3).unwrap(), doc3);
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 3);
    }

    #[test]
    fn test_page_compaction() {
        let mut page = create_test_page();
        
        // Insert several documents
        let doc1 = b"Doc1";
        let doc2 = b"Document2";
        let doc3 = b"Doc3";
        let doc4 = b"Document4";
        let doc5 = b"Doc5";
        
        let slot1 = PageLayout::insert_document(&mut page, doc1).unwrap();
        let slot2 = PageLayout::insert_document(&mut page, doc2).unwrap();
        let slot3 = PageLayout::insert_document(&mut page, doc3).unwrap();
        let slot4 = PageLayout::insert_document(&mut page, doc4).unwrap();
        let slot5 = PageLayout::insert_document(&mut page, doc5).unwrap();
        
        // Delete some documents to create fragmentation
        PageLayout::delete_document(&mut page, slot2).unwrap(); // Delete "Document2"
        PageLayout::delete_document(&mut page, slot4).unwrap(); // Delete "Document4"
        
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 3);
        
        // Compact the page
        PageLayout::compact_page(&mut page).unwrap();
        
        // Verify remaining documents are still accessible
        assert_eq!(PageLayout::get_document(&page, slot1).unwrap(), doc1);
        assert_eq!(PageLayout::get_document(&page, slot3).unwrap(), doc3);
        assert_eq!(PageLayout::get_document(&page, slot5).unwrap(), doc5);
        assert_eq!(PageLayout::get_document_count(&page).unwrap(), 3);
    }

    #[test]
    fn test_utilization_percentage() {
        let mut page = create_test_page();
        
        // Empty page should have 0% utilization
        let utilization = PageLayout::get_utilization_percentage(&page).unwrap();
        assert_eq!(utilization, 0.0);
        
        // Insert a document
        let doc_data = b"Test document for utilization";
        PageLayout::insert_document(&mut page, doc_data).unwrap();
        
        let utilization = PageLayout::get_utilization_percentage(&page).unwrap();
        assert!(utilization > 0.0);
        assert!(utilization <= 100.0);
    }

    #[test]
    fn test_large_document_storage() {
        let mut page = create_test_page();
        
        // Create a large document (but not too large for the page)
        let large_doc = vec![b'X'; 1000];
        
        let slot_id = PageLayout::insert_document(&mut page, &large_doc).unwrap();
        let retrieved_data = PageLayout::get_document(&page, slot_id).unwrap();
        
        assert_eq!(retrieved_data.len(), 1000);
        assert_eq!(retrieved_data, large_doc.as_slice());
    }

    #[test]
    fn test_empty_document_error() {
        let mut page = create_test_page();
        
        let result = PageLayout::insert_document(&mut page, &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_slot_access() {
        let page = create_test_page();
        
        // Try to access non-existent slot
        let result = PageLayout::get_document(&page, 999);
        assert!(result.is_err());
        
        // Try to delete non-existent slot
        let mut page = page;
        let result = PageLayout::delete_document(&mut page, 999);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_deleted_document() {
        let mut page = create_test_page();
        
        let doc_data = b"Document to delete and update";
        let slot_id = PageLayout::insert_document(&mut page, doc_data).unwrap();
        
        PageLayout::delete_document(&mut page, slot_id).unwrap();
        
        // Should fail to update deleted document
        let result = PageLayout::update_document(&mut page, slot_id, b"Updated data");
        assert!(result.is_err());
    }

    #[test]
    fn test_document_sizes() {
        let mut page = create_test_page();
        
        // Test various document sizes
        let sizes = [1, 10, 100, 500, 1000];
        
        for size in sizes {
            let doc = vec![b'A'; size];
            let slot_id = PageLayout::insert_document(&mut page, &doc).unwrap();
            let retrieved = PageLayout::get_document(&page, slot_id).unwrap();
            assert_eq!(retrieved.len(), size);
        }
    }
}
