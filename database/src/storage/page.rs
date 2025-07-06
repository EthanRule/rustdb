use crate::error::DatabaseError;
use std::mem;

// A page should be of a fixed size.
pub const PAGE_SIZE: usize = 8192;

// The type of the page, indicating what kind of data it stores.
// It's important to use a fixed-size representation for enums that are part of a data structure
// that needs a predictable size. `#[repr(u8)]` ensures the enum is stored as a single byte.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum PageType {
    Data = 0,
    Index = 1,
    Metadata = 2,
    Free = 3,
}

impl From<u8> for PageType {
    fn from(value: u8) -> Self {
        match value {
            0 => PageType::Data,
            1 => PageType::Index,
            2 => PageType::Metadata,
            3 => PageType::Free,
            // It's good practice to handle invalid values.
            _ => panic!("Invalid value for PageType: {}", value),
        }
    }
}

impl From<PageType> for u8 {
    fn from(value: PageType) -> u8 {
        value as u8
    }
}

// A Page is a fixed-size block of data as it would be on disk.
// The layout is a PageHeader followed by the page's content.
pub struct Page {
    data: [u8; PAGE_SIZE],
}

impl Page {
    /// Creates a new in-memory page.
    ///
    /// This function initializes a new page with a given page ID and type. It sets up the
    /// page header, calculates the initial free space, and computes a checksum for the
    /// page's initial state.
    pub fn new(page_id: u64, page_type: PageType) -> Self {
        let mut page = Page {
            data: [0u8; PAGE_SIZE],
        };

        // The free space is the total page size minus the space occupied by the header.
        // This is the amount of space available for storing tuples or other data.
        let free_space = (PAGE_SIZE - mem::size_of::<PageHeader>()) as u16;

        let header = PageHeader {
            page_id,
            page_type,
            free_space,
            checksum: 0, // Checksum is calculated after the header is written
        };
        
        page.write_header(&header);

        // Now that the header is written, calculate the initial checksum for the page
        // and update the checksum field in the header.
        let checksum = page.calculate_checksum();
        page.header_mut().checksum = checksum;

        page
    }

    /// Deserializes a page from a byte array.
    ///
    /// This function takes a raw byte array, creates a Page from it, and verifies its
    /// integrity by checking the checksum.
    pub fn from_bytes(data: [u8; PAGE_SIZE]) -> Result<Self, DatabaseError> {
        let page = Page { data };
        if !page.verify_checksum() {
            return Err(DatabaseError::InvalidChecksum);
        }
        Ok(page)
    }

    /// Serializes the page into a byte array for writing to disk.
    pub fn to_bytes(&self) -> [u8; PAGE_SIZE] {
        self.data
    }

    // A helper function to get an immutable reference to the header.
    // The header is located at the beginning of the page data.
    fn header(&self) -> &PageHeader {
        // This is safe because we know the header is at the start of the `data` array
        // and we control the layout.
        unsafe { &*(self.data.as_ptr() as *const PageHeader) }
    }

    // A helper function to get a mutable reference to the header.
    fn header_mut(&mut self) -> &mut PageHeader {
        unsafe { &mut *(self.data.as_mut_ptr() as *mut PageHeader) }
    }

    fn write_header(&mut self, header: &PageHeader) {
        let header_size = mem::size_of::<PageHeader>();
        let header_slice = unsafe {
            std::slice::from_raw_parts((header as *const PageHeader) as *const u8, header_size)
        };
        self.data[..header_size].copy_from_slice(header_slice);
    }

    /// Calculates the CRC32 checksum of the page.
    /// The checksum is calculated over the entire page data, but the checksum field
    /// in the header is temporarily treated as zero to ensure a consistent hash.
    pub fn calculate_checksum(&self) -> u32 {
        let mut hasher = crc32fast::Hasher::new();
        let header_size = mem::size_of::<PageHeader>();
        let checksum_offset = mem::offset_of!(PageHeader, checksum);

        // Hash the header part before the checksum field
        hasher.update(&self.data[..checksum_offset]);
        // Skip the checksum field by hashing zeros instead
        hasher.update(&[0u8; 4]);
        // Hash the rest of the header
        hasher.update(&self.data[checksum_offset + 4..header_size]);
        // Hash the rest of the page data
        hasher.update(&self.data[header_size..]);

        hasher.finalize()
    }

    /// Verifies the page's integrity by recalculating the checksum and comparing
    /// it with the one stored in the header.
    pub fn verify_checksum(&self) -> bool {
        self.calculate_checksum() == self.header().checksum
    }

    /// Returns the amount of free space on the page.
    pub fn get_free_space(&self) -> u16 {
        self.header().free_space
    }

    /// Updates the free space counter in the page header.
    /// This should be called whenever data is added to or removed from the page.
    pub fn update_free_space(&mut self, new_free_space: u16) {
        self.header_mut().free_space = new_free_space;
    }
}

// The page header contains metadata about the page.
// It's stored at the beginning of the page data.
// `#[repr(C)]` is important to ensure that the struct fields are laid out in memory
// in the order they are defined, with C-like padding. This gives us a predictable
// size and layout for serialization.
#[repr(C)]
struct PageHeader {
    page_id: u64,
    page_type: PageType,
    free_space: u16,
    // A checksum is used to detect data corruption.
    // It's calculated from the page's content.
    // When the page is read from disk, the checksum can be recalculated
    // and compared with the stored one to ensure the data is not corrupted.
    checksum: u32,
}

// This struct could hold metadata specific to certain page types.
// For example, for a B-Tree index page, it might store the level of the node in the tree.
// This would typically be part of the `data` field of the `Page`, not a separate field in `Page` struct.
#[allow(dead_code)]
struct PageMetadata {

}

// This enum was probably intended to be PageType.
// I have renamed it and moved it to the top of the file.
// The variants with associated data like `Metadata(PageMetadata)` and `Free(u16)`
// would make the enum's size variable, which is not suitable for direct inclusion
// in a fixed-size header. That data should be stored within the page's data area.

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_new_page() {
        let page_id = 1;
        let page_type = PageType::Data;
        let page = Page::new(page_id, page_type);
        let header = page.header();

        assert_eq!(header.page_id, page_id);
        assert_eq!(header.page_type, page_type);
        let expected_free_space = (PAGE_SIZE - mem::size_of::<PageHeader>()) as u16;
        assert_eq!(header.free_space, expected_free_space);
        assert!(page.verify_checksum());
    }

    #[test]
    fn test_page_serialization_deserialization() {
        let page_id = 2;
        let page_type = PageType::Index;
        let page = Page::new(page_id, page_type);

        // Serialize the page to bytes
        let bytes = page.to_bytes();

        // Deserialize back to a page
        let deserialized_page = Page::from_bytes(bytes).unwrap();

        assert_eq!(page.header().page_id, deserialized_page.header().page_id);
        assert_eq!(
            page.header().page_type,
            deserialized_page.header().page_type
        );
        assert_eq!(
            page.header().free_space,
            deserialized_page.header().free_space
        );
        assert_eq!(
            page.header().checksum,
            deserialized_page.header().checksum
        );
        assert!(deserialized_page.verify_checksum());
    }

    #[test]
    fn test_invalid_checksum() {
        let page_id = 3;
        let page_type = PageType::Metadata;
        let page = Page::new(page_id, page_type);

        let mut bytes = page.to_bytes();
        // Corrupt the data
        bytes[mem::size_of::<PageHeader>() + 10] ^= 0xff;

        let result = Page::from_bytes(bytes);
        assert!(matches!(result, Err(DatabaseError::InvalidChecksum)));
    }

    #[test]
    fn test_update_and_get_free_space() {
        let mut page = Page::new(4, PageType::Free);
        let initial_free_space = page.get_free_space();
        assert_eq!(
            initial_free_space,
            (PAGE_SIZE - mem::size_of::<PageHeader>()) as u16
        );

        let new_free_space = initial_free_space - 100;
        page.update_free_space(new_free_space);
        assert_eq!(page.get_free_space(), new_free_space);
    }
}

