use database::{Document, Value};

#[test]
fn test_document_field_iteration() {
    let mut doc = Document::new();
    
    // Add some test fields
    doc.set("name", Value::String("Alice Johnson".to_string()));
    doc.set("age", Value::I32(28));
    doc.set("active", Value::Bool(true));
    doc.set("salary", Value::F64(75000.50));
    
    // Test iteration over fields
    let mut field_count = 0;
    let mut found_fields = std::collections::HashSet::new();
    
    for (key, value) in doc.iter() {
        field_count += 1;
        found_fields.insert(key.clone());
        
        match key.as_str() {
            "name" => {
                if let Value::String(s) = value {
                    assert_eq!(s, "Alice Johnson");
                } else {
                    panic!("Expected string value for name");
                }
            }
            "age" => {
                if let Value::I32(i) = value {
                    assert_eq!(*i, 28);
                } else {
                    panic!("Expected i32 value for age");
                }
            }
            "active" => {
                if let Value::Bool(b) = value {
                    assert!(b);
                } else {
                    panic!("Expected bool value for active");
                }
            }
            "salary" => {
                if let Value::F64(f) = value {
                    assert_eq!(*f, 75000.50);
                } else {
                    panic!("Expected f64 value for salary");
                }
            }
            _ => panic!("Unexpected field: {}", key),
        }
    }
    
    assert_eq!(field_count, 4);
    assert!(found_fields.contains("name"));
    assert!(found_fields.contains("age"));
    assert!(found_fields.contains("active"));
    assert!(found_fields.contains("salary"));
    
    // Test keys iterator
    let keys: Vec<_> = doc.keys().collect();
    assert_eq!(keys.len(), 4);
    
    // Test values iterator
    let values: Vec<_> = doc.values().collect();
    assert_eq!(values.len(), 4);
    
    // Test document properties
    assert!(!doc.is_empty());
    assert_eq!(doc.len(), 4);
    
    println!("✅ Document field iteration test passed!");
}

#[test]
fn test_empty_document_iteration() {
    let doc = Document::new();
    
    // Test empty document
    assert!(doc.is_empty());
    assert_eq!(doc.len(), 0);
    
    let mut field_count = 0;
    for (_key, _value) in doc.iter() {
        field_count += 1;
    }
    assert_eq!(field_count, 0);
    
    println!("✅ Empty document iteration test passed!");
}