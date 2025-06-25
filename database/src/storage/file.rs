use crate::error::DatabaseError;
use crate::storage::page::{Page, PAGE_SIZE};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

const DATABASE_VERSION: u8 = 1;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct FileHeader {
    version: u8,
    page_count: u64,
    /// Reserved space for future metadata.
    #[serde(with = "u8_64_serde")]
    metadata: [u8; 64],
}

mod u8_64_serde {
    use serde::{
        de::{self, Visitor},
        Deserializer, Serializer,
    };
    use std::convert::TryInto;

    pub fn serialize<S>(array: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(array)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ByteArrayVisitor;

        impl<'de> Visitor<'de> for ByteArrayVisitor {
            type Value = [u8; 64];

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a byte array of length 64")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                v.try_into().map_err(|_| {
                    E::custom(format!(
                        "expected byte array of length 64, but got {}",
                        v.len()
                    ))
                })
            }
        }

        deserializer.deserialize_bytes(ByteArrayVisitor)
    }
}

impl FileHeader {
    fn new() -> Self {
        Self {
            version: DATABASE_VERSION,
            page_count: 0,
            metadata: [0; 64],
        }
    }

    /// Returns the serialized size of the header.
    fn size() -> u64 {
        bincode::serialized_size(&Self::new()).unwrap()
    }
}

pub struct DatabaseFile {
    file: File,
    header: FileHeader,
}

impl DatabaseFile {
    pub fn create(path: &Path) -> Result<Self, DatabaseError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)?;

        // Lock the file exclusively to prevent other processes from using it.
        file.try_lock_exclusive()
            .map_err(|e| DatabaseError::Io(e.into()))?;

        let header = FileHeader::new();
        let mut db_file = Self { file, header };

        db_file.write_header()?;
        db_file.sync()?;

        Ok(db_file)
    }

    /// Opens an existing database file.
    ///
    /// This will open the file, acquire an exclusive lock, and read and validate
    /// the file header.
    pub fn open(path: &Path) -> Result<Self, DatabaseError> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;

        // Lock the file exclusively.
        file.try_lock_exclusive()
            .map_err(|e| DatabaseError::Io(e.into()))?;

        let mut db_file = Self {
            file,
            // Header will be read from file.
            header: FileHeader::new(),
        };

        db_file.read_header()?;

        if db_file.header.version != DATABASE_VERSION {
            return Err(DatabaseError::Storage(format!(
                "Incompatible database version. Expected {}, found {}",
                DATABASE_VERSION, db_file.header.version
            )));
        }

        Ok(db_file)
    }

    /// Reads the file header from disk.
    fn read_header(&mut self) -> Result<(), DatabaseError> {
        let mut buffer = vec![0; FileHeader::size() as usize];
        self.file.seek(SeekFrom::Start(0))?;
        self.file.read_exact(&mut buffer)?;
        self.header = bincode::deserialize(&buffer).map_err(DatabaseError::Bincode)?;
        Ok(())
    }

    /// Writes the file header to disk.
    fn write_header(&mut self) -> Result<(), DatabaseError> {
        let buffer = bincode::serialize(&self.header).map_err(DatabaseError::Bincode)?;
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&buffer)?;
        Ok(())
    }

    /// Reads a specific page from the disk.
    pub fn read_page(&mut self, page_id: u64) -> Result<Page, DatabaseError> {
        if page_id >= self.header.page_count {
            return Err(DatabaseError::Storage(format!(
                "Attempted to read non-existent page {}",
                page_id
            )));
        }
        let offset = FileHeader::size() + page_id * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;

        let mut buffer = [0u8; PAGE_SIZE];
        self.file.read_exact(&mut buffer)?;

        Page::from_bytes(buffer)
    }

    /// Writes a page to the disk at a specific page ID.
    pub fn write_page(&mut self, page_id: u64, page: &Page) -> Result<(), DatabaseError> {
        if page_id >= self.header.page_count {
            return Err(DatabaseError::Storage(format!(
                "Attempted to write to non-existent page {}",
                page_id
            )));
        }
        let offset = FileHeader::size() + page_id * PAGE_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&page.to_bytes())?;
        Ok(())
    }

    /// Allocates a new page in the database file.
    ///
    /// This extends the file size and increments the page count in the header.
    /// Returns the new page ID.
    pub fn allocate_page(&mut self) -> Result<u64, DatabaseError> {
        let new_page_id = self.header.page_count;
        let new_page_count = new_page_id + 1;
        let new_file_size = FileHeader::size() + new_page_count * PAGE_SIZE as u64;

        // Extend the file size
        self.file.set_len(new_file_size)?;

        // Update header
        self.header.page_count = new_page_count;
        self.write_header()?;

        Ok(new_page_id)
    }

    /// Flushes all in-memory changes to the disk.
    pub fn sync(&self) -> Result<(), DatabaseError> {
        self.file.sync_all()?;
        Ok(())
    }

    /// Returns the number of pages in the file.
    pub fn page_count(&self) -> u64 {
        self.header.page_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::page::PageType;
    use tempfile;

    #[test]
    fn test_create_and_open_db_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.db");

        // Create
        {
            let db_file = DatabaseFile::create(&path).unwrap();
            assert_eq!(db_file.page_count(), 0);
        }

        // Open
        {
            let db_file = DatabaseFile::open(&path).unwrap();
            assert_eq!(db_file.header.version, DATABASE_VERSION);
            assert_eq!(db_file.page_count(), 0);
        }
    }

    #[test]
    fn test_allocate_and_write_read_page() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.db");
        let mut db_file = DatabaseFile::create(&path).unwrap();

        // Allocate a page
        let page_id = db_file.allocate_page().unwrap();
        assert_eq!(page_id, 0);
        assert_eq!(db_file.page_count(), 1);

        // Create a new page and write it
        let page_to_write = Page::new(page_id, PageType::Data);
        db_file.write_page(page_id, &page_to_write).unwrap();

        // Read the page back
        let page_read = db_file.read_page(page_id).unwrap();

        // Verify
        assert_eq!(page_to_write.to_bytes(), page_read.to_bytes());
        assert!(page_read.verify_checksum());
    }

    #[test]
    fn test_multiple_pages() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.db");
        let mut db_file = DatabaseFile::create(&path).unwrap();

        let num_pages = 5;
        let mut pages = Vec::new();

        for i in 0..num_pages {
            let page_id = db_file.allocate_page().unwrap();
            assert_eq!(page_id, i);
            let page = Page::new(page_id, PageType::Index);
            db_file.write_page(page_id, &page).unwrap();
            pages.push(page);
        }

        assert_eq!(db_file.page_count(), num_pages);

        for i in 0..num_pages {
            let page_read = db_file.read_page(i).unwrap();
            assert_eq!(pages[i as usize].to_bytes(), page_read.to_bytes());
        }
    }

    #[test]
    fn test_read_non_existent_page() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.db");
        let mut db_file = DatabaseFile::create(&path).unwrap();

        let result = db_file.read_page(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_sync() {
        let temp_dir = tempfile::tempdir().unwrap();
        let path = temp_dir.path().join("test.db");
        let db_file = DatabaseFile::create(&path).unwrap();
        assert!(db_file.sync().is_ok());
    }
}
