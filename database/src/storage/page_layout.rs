use crate::storage::page::Page;
use crate::error::{DatabaseError};
use std::collections::HashMap;
use std::collections::LinkedList;

pub type SlotId = u16;

const SLOT_DIRECTORY_OFFSET: usize = 8184;
const MAX_SLOTS_PER_PAGE: u16 = 1000;
const SLOT_SIZE: usize = 4;
const TOMBSTONE_MARKER: u16 = 0xFFFF;

struct ListNode {
    val: bool,
}

pub struct PageLayout {
    page_header: u8,
    free_space: LinkedList<ListNode>,

    
    // 
}

pub struct SlotDirectoryHeader {
    slot_count: u16,
    slots: Vec<(u16, u16)>,
}

impl PageLayout {
    pub fn track_free_space(page: Page) {
        
    }
}
