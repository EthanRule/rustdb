use database::*;

#[test]
fn test_document_new_and_set() {
    let mut doc = Document::new();
    doc.set("name", Value::String("Alice".to_string()));
    doc.set("age", Value::I32(30));

    assert_eq!(
        *doc.get("name").unwrap(),
        Value::String("Alice".to_string())
    );
    assert_eq!(*doc.get("age").unwrap(), Value::I32(30));
}

#[test]
fn test_document_remove() {
    let mut doc = Document::new();
    doc.set("name", Value::String("Alice".to_string()));
    doc.set("age", Value::I32(30));

    assert!(doc.remove("name").is_some());
    assert!(doc.get("name").is_none());
    assert_eq!(*doc.get("age").unwrap(), Value::I32(30));
}

// Additional integration tests:

#[test]
fn test_document_overwrite_value() {
    let mut doc = Document::new();
    doc.set("counter", Value::I32(1));
    doc.set("counter", Value::I32(2));

    assert_eq!(*doc.get("counter").unwrap(), Value::I32(2));
}

#[test]
fn test_document_get_path_simple() {
    let mut doc = Document::new();
    let mut nested = std::collections::BTreeMap::new();
    nested.insert("inner".to_string(), Value::Bool(true));
    doc.set("outer", Value::Object(nested));

    assert_eq!(doc.get_path("outer.inner"), Some(&Value::Bool(true)));
}

#[test]
fn test_document_get_path_missing() {
    let doc = Document::new();
    assert!(doc.get_path("not.there").is_none());
}

#[test]
fn test_document_from_json_valid() {
    let json = r#"{"x": 42, "y": false}"#;
    let doc = Document::from_json(json).unwrap();
    assert_eq!(doc.get("x"), Some(&Value::I32(42)));
    assert_eq!(doc.get("y"), Some(&Value::Bool(false)));
}

#[test]
fn test_document_from_json_invalid() {
    let json = "not valid json";
    assert!(Document::from_json(json).is_err());
}
