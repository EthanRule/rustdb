// 🎯 Real-World Analogy
// Think of it like a library with a smart lending system:

// Pages = Books on shelves
// Buffer Pool = Reading room (limited space, keeps popular books)
// Pinning = Checking out a book to your table
// Slots = Page numbers within each book
// Dirty = You wrote notes in the margins (needs to be saved)
// Unpinning = Returning the book (clean or with notes to be filed)
// TODO: Consider adding a tombstone Vacuum

use crate::{
    document::bson::{deserialize_document, serialize_document},
    storage::{buffer_pool::BufferPool, file::DatabaseFile, page_layout::PageLayout},
    Document,
};
use anyhow::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct DocumentId {
    page_id: u64,
    slot_id: u16,
}

impl DocumentId {
    /// Create a new DocumentId
    pub fn new(page_id: u64, slot_id: u16) -> Self {
        Self { page_id, slot_id }
    }

    /// Get the page ID where the document is stored
    pub fn page_id(&self) -> u64 {
        self.page_id
    }

    /// Get the slot ID within the page where the document is stored
    pub fn slot_id(&self) -> u16 {
        self.slot_id
    }
}

pub struct StorageEngine {
    pub database_file: DatabaseFile,
    buffer_pool: BufferPool,
}

impl StorageEngine {
    pub fn new(database_path: &Path, buffer_pool_size: usize) -> Result<Self> {
        let database_file = DatabaseFile::open(database_path)?;
        let buffer_pool = BufferPool::new(buffer_pool_size);
        Ok(Self {
            database_file,
            buffer_pool,
        })
    }

    pub fn insert_document(&mut self, document: &Document) -> Result<DocumentId> {
        // 1. Serialize the document to BSON bytes
        let document_bytes = serialize_document(document)
            .map_err(|e| anyhow::anyhow!("Failed to serialize document: {}", e))?;
        let document_size = document_bytes.len();

        // 2. Try to find an existing page with enough free space
        let page_ids = self.buffer_pool.get_all_page_ids();
        for page_id in page_ids {
            // Pin the page to get mutable access
            if let Ok(page) = self.buffer_pool.pin_page(page_id, &mut self.database_file) {
                let free_space = page.get_free_space() as usize;

                // Check if document can fit in this page
                if document_size <= free_space {
                    // Insert the document using PageLayout
                    match PageLayout::insert_document(page, &document_bytes) {
                        Ok(slot_id) => {
                            // Mark the page as dirty and unpin it
                            self.buffer_pool.unpin_page(page_id, true); // true = is_dirty
                            return Ok(DocumentId {
                                page_id: page_id,
                                slot_id,
                            });
                        }
                        Err(_) => {
                            // Failed to insert, unpin the page without marking dirty
                            self.buffer_pool.unpin_page(page_id, false);
                            continue;
                        }
                    }
                }
                // Page doesn't have enough space, unpin it
                self.buffer_pool.unpin_page(page_id, false);
            }
        }

        // Page doesen't exist, or not enough space? Allocate more space and insert a fresh page.
        let new_page_id = self.database_file.allocate_page()?;

        let page = self
            .buffer_pool
            .pin_page(new_page_id, &mut self.database_file)?;

        let slot_id = PageLayout::insert_document(page, &document_bytes)?;

        self.buffer_pool.unpin_page(new_page_id, true);

        Ok(DocumentId {
            page_id: new_page_id,
            slot_id: slot_id,
        })
    }

    pub fn get_document(&mut self, document_id: &DocumentId) -> Result<Document> {
        let page = self
            .buffer_pool
            .pin_page(document_id.page_id, &mut self.database_file)?;
        let document_bytes = PageLayout::get_document(page, document_id.slot_id)?;
        self.buffer_pool.unpin_page(document_id.page_id(), false);

        Ok(deserialize_document(&document_bytes)?)
    }

    pub fn update_document(
        &mut self,
        document_id: &DocumentId,
        new_document: &Document,
    ) -> Result<DocumentId> {
        // 1. Serialize the new document
        let new_document_bytes = serialize_document(new_document)
            .map_err(|e| anyhow::anyhow!("Failed to serialize document: {}", e))?;
        let new_size = new_document_bytes.len();

        // 2. Pin the original page
        let page = self
            .buffer_pool
            .pin_page(document_id.page_id, &mut self.database_file)?;

        // 3. Get the old document size for comparison
        let old_document_bytes = PageLayout::get_document(page, document_id.slot_id)?;
        let old_size = old_document_bytes.len();

        // 4. Check if new document fits in the same slot
        if new_size <= old_size {
            // Case 1: New document fits in same slot (in-place update)
            PageLayout::update_document(page, document_id.slot_id, &new_document_bytes)?;
            self.buffer_pool.unpin_page(document_id.page_id, true); // Mark as dirty
            Ok(*document_id) // Return same DocumentId
        } else {
            // Case 2: New document doesn't fit, need to relocate

            // First, check if the same page has enough free space
            let available_space = page.get_free_space() as usize;
            if new_size <= available_space + old_size {
                // Can fit on same page after deleting old document
                PageLayout::delete_document(page, document_id.slot_id)?;
                let new_slot_id = PageLayout::insert_document(page, &new_document_bytes)?;
                self.buffer_pool.unpin_page(document_id.page_id, true);

                Ok(DocumentId::new(document_id.page_id, new_slot_id))
            } else {
                // Need to move to different page

                // Mark old slot as deleted (tombstone)
                PageLayout::delete_document(page, document_id.slot_id)?;
                self.buffer_pool.unpin_page(document_id.page_id, true);

                // Insert into new location (reuse insert_document logic)
                self.insert_document_internal(&new_document_bytes)
            }
        }
    }

    pub fn delete_document(&mut self, document_id: &DocumentId) -> Result<()> {
        // 1. Pin the page containing the document
        let page = self
            .buffer_pool
            .pin_page(document_id.page_id, &mut self.database_file)?;

        // 2. Mark the document slot as deleted (tombstone)
        PageLayout::delete_document(page, document_id.slot_id)?;

        // 3. Mark page as dirty and unpin
        self.buffer_pool.unpin_page(document_id.page_id, true);

        Ok(())
    }

    // Helper function to avoid code duplication
    fn insert_document_internal(&mut self, document_bytes: &[u8]) -> Result<DocumentId> {
        let document_size = document_bytes.len();

        // Try to find an existing page with enough free space
        let page_ids = self.buffer_pool.get_all_page_ids();
        for page_id in page_ids {
            if let Ok(page) = self.buffer_pool.pin_page(page_id, &mut self.database_file) {
                let free_space = page.get_free_space() as usize;

                if document_size <= free_space {
                    match PageLayout::insert_document(page, document_bytes) {
                        Ok(slot_id) => {
                            self.buffer_pool.unpin_page(page_id, true);
                            return Ok(DocumentId::new(page_id, slot_id));
                        }
                        Err(_) => {
                            self.buffer_pool.unpin_page(page_id, false);
                            continue;
                        }
                    }
                }
                self.buffer_pool.unpin_page(page_id, false);
            }
        }

        // Need a new page
        let new_page_id = self.database_file.allocate_page()?;
        let page = self
            .buffer_pool
            .pin_page(new_page_id, &mut self.database_file)?;
        let slot_id = PageLayout::insert_document(page, document_bytes)?;
        self.buffer_pool.unpin_page(new_page_id, true);

        Ok(DocumentId::new(new_page_id, slot_id))
    }
}
