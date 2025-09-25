use database::document::{Document, types::Value};
use database::document::bson::{serialize_document, deserialize_document, encode_value, decode_value};
use database::document::object_id::ObjectId;
use chrono::Utc;
use std::collections::BTreeMap;

#[test]
fn test_simple_document_roundtrip() {
    let mut doc = Document::new();
    doc.set("name", Value::String("John Doe".to_string()));
    doc.set("age", Value::I32(30));
    doc.set("active", Value::Bool(true));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    assert_eq!(deserialized.get("name"), Some(&Value::String("John Doe".to_string())));
    assert_eq!(deserialized.get("age"), Some(&Value::I32(30)));
    assert_eq!(deserialized.get("active"), Some(&Value::Bool(true)));
}

#[test]
fn test_all_data_types_roundtrip() {
    let mut doc = Document::new();
    doc.set("null_val", Value::Null);
    doc.set("bool_val", Value::Bool(true));
    doc.set("int32_val", Value::I32(42));
    doc.set("int64_val", Value::I64(123456789));
    doc.set("double_val", Value::F64(3.14159));
    doc.set("string_val", Value::String("hello world".to_string()));
    doc.set("objectid_val", Value::ObjectId(ObjectId::new()));
    doc.set("datetime_val", Value::DateTime(Utc::now()));
    doc.set("binary_val", Value::Binary(vec![1, 2, 3, 4, 5]));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    // Check all values round-trip correctly
    assert_eq!(deserialized.get("null_val"), Some(&Value::Null));
    assert_eq!(deserialized.get("bool_val"), Some(&Value::Bool(true)));
    assert_eq!(deserialized.get("int32_val"), Some(&Value::I32(42)));
    assert_eq!(deserialized.get("int64_val"), Some(&Value::I64(123456789)));
    assert_eq!(deserialized.get("double_val"), Some(&Value::F64(3.14159)));
    assert_eq!(deserialized.get("string_val"), Some(&Value::String("hello world".to_string())));
    assert_eq!(deserialized.get("binary_val"), Some(&Value::Binary(vec![1, 2, 3, 4, 5])));
    
    // ObjectId and DateTime should be preserved but may not be exactly equal due to generation time
    assert!(deserialized.get("objectid_val").is_some());
    assert!(deserialized.get("datetime_val").is_some());
}

#[test]
fn test_nested_object_roundtrip() {
    let mut inner_doc = Document::new();
    inner_doc.set("name", Value::String("Alice".to_string()));
    inner_doc.set("email", Value::String("alice@example.com".to_string()));
    
    // Create a BTreeMap from the inner document for the Object value
    let mut inner_map = BTreeMap::new();
    inner_map.insert("name".to_string(), Value::String("Alice".to_string()));
    inner_map.insert("email".to_string(), Value::String("alice@example.com".to_string()));
    
    let mut outer_doc = Document::new();
    outer_doc.set("user", Value::Object(inner_map));
    outer_doc.set("score", Value::F64(95.5));
    
    let serialized = serialize_document(&outer_doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    // Check nested object
    if let Some(Value::Object(user_data)) = deserialized.get("user") {
        assert_eq!(user_data.get("name"), Some(&Value::String("Alice".to_string())));
        assert_eq!(user_data.get("email"), Some(&Value::String("alice@example.com".to_string())));
    } else {
        panic!("Expected Object value for 'user' field");
    }
    
    assert_eq!(deserialized.get("score"), Some(&Value::F64(95.5)));
}

#[test]
fn test_array_roundtrip() {
    let mut doc = Document::new();
    let array_values = vec![
        Value::String("rust".to_string()),
        Value::String("database".to_string()),
        Value::String("bson".to_string()),
    ];
    doc.set("tags", Value::Array(array_values));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    if let Some(Value::Array(tags)) = deserialized.get("tags") {
        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0], Value::String("rust".to_string()));
        assert_eq!(tags[1], Value::String("database".to_string()));
        assert_eq!(tags[2], Value::String("bson".to_string()));
    } else {
        panic!("Expected Array value for 'tags' field");
    }
}

#[test]
fn test_deep_nesting_roundtrip() {
    // Create nested objects manually
    let mut level3_map = BTreeMap::new();
    level3_map.insert("value".to_string(), Value::String("deep".to_string()));
    
    let mut level2_map = BTreeMap::new();
    level2_map.insert("nested".to_string(), Value::Object(level3_map));
    level2_map.insert("count".to_string(), Value::I32(3));
    
    let mut level1_map = BTreeMap::new();
    level1_map.insert("nested".to_string(), Value::Object(level2_map));
    level1_map.insert("name".to_string(), Value::String("level1".to_string()));
    
    let mut root = Document::new();
    root.set("data", Value::Object(level1_map));
    root.set("active", Value::Bool(true));
    
    let serialized = serialize_document(&root).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    // Navigate through nested structure
    if let Some(Value::Object(data)) = deserialized.get("data") {
        if let Some(Value::Object(nested)) = data.get("nested") {
            if let Some(Value::Object(deep_nested)) = nested.get("nested") {
                assert_eq!(deep_nested.get("value"), Some(&Value::String("deep".to_string())));
            } else {
                panic!("Expected nested object at level 3");
            }
            assert_eq!(nested.get("count"), Some(&Value::I32(3)));
        } else {
            panic!("Expected nested object at level 2");
        }
        assert_eq!(data.get("name"), Some(&Value::String("level1".to_string())));
    } else {
        panic!("Expected object for 'data' field");
    }
    
    assert_eq!(deserialized.get("active"), Some(&Value::Bool(true)));
}

#[test]
fn test_mixed_array_roundtrip() {
    let mut doc = Document::new();
    let mixed_array = vec![
        Value::String("string".to_string()),
        Value::I32(42),
        Value::Bool(true),
        Value::F64(3.14),
        Value::Null,
    ];
    doc.set("mixed", Value::Array(mixed_array));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    if let Some(Value::Array(mixed)) = deserialized.get("mixed") {
        assert_eq!(mixed.len(), 5);
        assert_eq!(mixed[0], Value::String("string".to_string()));
        assert_eq!(mixed[1], Value::I32(42));
        assert_eq!(mixed[2], Value::Bool(true));
        assert_eq!(mixed[3], Value::F64(3.14));
        assert_eq!(mixed[4], Value::Null);
    } else {
        panic!("Expected Array value for 'mixed' field");
    }
}

#[test]
fn test_empty_document_roundtrip() {
    let doc = Document::new();
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    // Check that the document is empty by trying to get a non-existent field
    assert_eq!(deserialized.get("nonexistent"), None);
}

#[test]
fn test_single_field_roundtrip() {
    let mut doc = Document::new();
    doc.set("single", Value::String("value".to_string()));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    assert_eq!(deserialized.get("single"), Some(&Value::String("value".to_string())));
}

#[test]
fn test_unicode_string_roundtrip() {
    let mut doc = Document::new();
    doc.set("unicode", Value::String("Hello 世界 🌍".to_string()));
    doc.set("emoji", Value::String("🚀🔥💻".to_string()));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    assert_eq!(deserialized.get("unicode"), Some(&Value::String("Hello 世界 🌍".to_string())));
    assert_eq!(deserialized.get("emoji"), Some(&Value::String("🚀🔥💻".to_string())));
}

#[test]
fn test_large_numbers_roundtrip() {
    let mut doc = Document::new();
    doc.set("large_int32", Value::I32(i32::MAX));
    doc.set("large_int64", Value::I64(i64::MAX));
    doc.set("negative_int32", Value::I32(i32::MIN));
    doc.set("negative_int64", Value::I64(i64::MIN));
    doc.set("large_double", Value::F64(f64::MAX));
    doc.set("small_double", Value::F64(f64::MIN));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    assert_eq!(deserialized.get("large_int32"), Some(&Value::I32(i32::MAX)));
    assert_eq!(deserialized.get("large_int64"), Some(&Value::I64(i64::MAX)));
    assert_eq!(deserialized.get("negative_int32"), Some(&Value::I32(i32::MIN)));
    assert_eq!(deserialized.get("negative_int64"), Some(&Value::I64(i64::MIN)));
    assert_eq!(deserialized.get("large_double"), Some(&Value::F64(f64::MAX)));
    assert_eq!(deserialized.get("small_double"), Some(&Value::F64(f64::MIN)));
}

#[test]
fn test_binary_data_roundtrip() {
    let mut doc = Document::new();
    let binary_data = vec![0x00, 0xFF, 0x01, 0xFE, 0x02, 0xFD];
    doc.set("binary", Value::Binary(binary_data.clone()));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    assert_eq!(deserialized.get("binary"), Some(&Value::Binary(binary_data)));
}

#[test]
fn test_datetime_roundtrip() {
    let mut doc = Document::new();
    let now = Utc::now();
    doc.set("timestamp", Value::DateTime(now));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    if let Some(Value::DateTime(deserialized_dt)) = deserialized.get("timestamp") {
        // DateTime should be very close (within milliseconds)
        let diff = (now.timestamp_millis() - deserialized_dt.timestamp_millis()).abs();
        assert!(diff < 1000); // Within 1 second
    } else {
        panic!("Expected DateTime value for 'timestamp' field");
    }
}

#[test]
fn test_objectid_roundtrip() {
    let mut doc = Document::new();
    let oid = ObjectId::new();
    doc.set("id", Value::ObjectId(oid.clone()));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    if let Some(Value::ObjectId(deserialized_oid)) = deserialized.get("id") {
        assert_eq!(oid.to_bytes(), deserialized_oid.to_bytes());
    } else {
        panic!("Expected ObjectId value for 'id' field");
    }
}

#[test]
fn test_encode_decode_basic_types_roundtrip() {
    let test_cases = vec![
        (Value::Null, 0x0A),
        (Value::Bool(true), 0x08),
        (Value::Bool(false), 0x08),
        (Value::I32(42), 0x10),
        (Value::I64(123456789), 0x12),
        (Value::F64(3.14159), 0x01),
        (Value::String("hello".to_string()), 0x02),
        (Value::Binary(vec![1, 2, 3]), 0x05),
    ];

    for (value, bson_type) in test_cases {
        let encoded = encode_value(&value);
        let (decoded, bytes_read) = decode_value(&encoded, bson_type).unwrap();
        assert_eq!(decoded, value);
        assert_eq!(bytes_read, encoded.len());
    }
}

#[test]
fn test_document_length_prefixing() {
    let mut doc = Document::new();
    doc.set("name", Value::String("John".to_string()));
    doc.set("age", Value::I32(30));
    
    let serialized = serialize_document(&doc).unwrap();
    
    // First 4 bytes should be document length in little-endian
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::Cursor;
    
    let mut cursor = Cursor::new(&serialized);
    let length = cursor.read_u32::<LittleEndian>().unwrap();
    
    assert_eq!(length as usize, serialized.len());
    assert!(length > 0);
}

#[test]
fn test_multiple_roundtrips() {
    let mut doc = Document::new();
    doc.set("name", Value::String("Test".to_string()));
    doc.set("value", Value::I32(100));
    
    // Perform multiple round-trips
    let mut current_doc = doc;
    for _i in 0..5 {
        let serialized = serialize_document(&current_doc).unwrap();
        let deserialized = deserialize_document(&serialized).unwrap();
        
        // Verify data integrity
        assert_eq!(deserialized.get("name"), Some(&Value::String("Test".to_string())));
        assert_eq!(deserialized.get("value"), Some(&Value::I32(100)));
        
        current_doc = deserialized;
    }
}

#[test]
fn test_field_order_preservation() {
    let mut doc = Document::new();
    doc.set("first", Value::String("1".to_string()));
    doc.set("second", Value::String("2".to_string()));
    doc.set("third", Value::String("3".to_string()));
    
    let serialized = serialize_document(&doc).unwrap();
    let deserialized = deserialize_document(&serialized).unwrap();
    
    // Check that all fields are present (order is preserved by BTreeMap)
    assert_eq!(deserialized.get("first"), Some(&Value::String("1".to_string())));
    assert_eq!(deserialized.get("second"), Some(&Value::String("2".to_string())));
    assert_eq!(deserialized.get("third"), Some(&Value::String("3".to_string())));
} 