use database::{storage::storage_engine::StorageEngine, Document, Value};
use tempfile::tempdir;

fn create_engine() -> (StorageEngine, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");

    let _db_file = database::storage::file::DatabaseFile::create(&db_path)
        .expect("Failed to create database file");
    drop(_db_file);

    let engine = StorageEngine::new(&db_path, 10).expect("Failed to create storage engine");
    (engine, temp_dir)
}

fn make_doc(name: &str) -> Document {
    let mut doc = Document::new();
    doc.set("name", Value::String(name.to_string()));
    doc
}

#[test]
fn test_vacuum_returns_zero_when_no_deletions() {
    let (mut engine, _dir) = create_engine();

    for i in 0..5 {
        engine
            .insert_document(&make_doc(&format!("doc_{}", i)))
            .expect("insert failed");
    }

    let pages_cleaned = engine.vacuum().expect("vacuum failed");
    assert_eq!(pages_cleaned, 0);
}

#[test]
fn test_vacuum_reclaims_space_after_deletions() {
    let (mut engine, _dir) = create_engine();

    let mut ids = vec![];
    for i in 0..10 {
        let id = engine
            .insert_document(&make_doc(&format!("doc_{}", i)))
            .expect("insert failed");
        ids.push(id);
    }

    // Delete half the documents
    for id in ids.iter().take(5) {
        engine.delete_document(id).expect("delete failed");
    }

    let pages_cleaned = engine.vacuum().expect("vacuum failed");
    assert!(pages_cleaned > 0, "expected at least one page to be compacted");
}

#[test]
fn test_vacuum_on_empty_database() {
    let (mut engine, _dir) = create_engine();
    let pages_cleaned = engine.vacuum().expect("vacuum failed");
    assert_eq!(pages_cleaned, 0);
}

#[test]
fn test_documents_readable_after_vacuum() {
    let (mut engine, _dir) = create_engine();

    let mut ids = vec![];
    for i in 0..6 {
        let id = engine
            .insert_document(&make_doc(&format!("doc_{}", i)))
            .expect("insert failed");
        ids.push(id);
    }

    // Delete every other document
    for id in ids.iter().step_by(2) {
        engine.delete_document(id).expect("delete failed");
    }

    engine.vacuum().expect("vacuum failed");

    // Surviving documents should still be readable
    for id in ids.iter().skip(1).step_by(2) {
        let doc = engine.get_document(id).expect("get after vacuum failed");
        assert!(doc.get("name").is_some());
    }
}

#[test]
fn test_vacuum_idempotent() {
    let (mut engine, _dir) = create_engine();

    let mut ids = vec![];
    for i in 0..6 {
        let id = engine
            .insert_document(&make_doc(&format!("doc_{}", i)))
            .expect("insert failed");
        ids.push(id);
    }

    for id in ids.iter().take(3) {
        engine.delete_document(id).expect("delete failed");
    }

    let first = engine.vacuum().expect("first vacuum failed");
    let second = engine.vacuum().expect("second vacuum failed");

    assert!(first > 0);
    assert_eq!(second, 0, "second vacuum should find nothing to compact");
}
