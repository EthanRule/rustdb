pub mod object_id;
pub mod types;
pub mod bson;
pub mod validator;

use crate::document::object_id::ObjectId;
use crate::document::types::Value;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// This file defines the Document struct used for storing a BTreeMap of <String, Value> pairs.
// To see examples of Document struct usages, scroll to the bottom.

const MAX_DOCUMENT_SIZE: usize = 16 * 1024 * 1024; // 16mb
const MAX_NAME_LENGTH: usize = 100; // 100 chars

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    data: BTreeMap<String, Value>,
    id: Value,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Document {
            data: BTreeMap::<String, Value>::new(),
            id: Value::ObjectId(ObjectId::new()),
        }
    }

    pub fn with_id(id: ObjectId) -> Self {
        Document {
            data: BTreeMap::new(),
            id: Value::ObjectId(id),
        }
    }

    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        let map: BTreeMap<String, serde_json::Value> = serde_json::from_str(input)?;
        let data = map
            .into_iter()
            .map(|(k, v)| (k, Value::from_json_value(v)))
            .collect();
        Ok(Document {
            data,
            id: Value::ObjectId(ObjectId::new()),
        })
    }

    pub fn get(&self, input: &str) -> Option<&Value> {
        self.data.get(input)
    }

    pub fn set<S: Into<String>>(&mut self, input: S, val: Value) {
        self.data.insert(input.into(), val);
    }

    pub fn remove(&mut self, input: &str) -> Option<Value> {
        self.data.remove(input)
    }

    pub fn get_path(&self, input: &str) -> Option<&Value> {
        let mut cur;

        let mut iter = input.split('.');

        if let Some(first) = iter.next() {
            cur = self.data.get(first);
        } else {
            return None;
        }

        for key in iter {
            match cur {
                Some(Value::Object(map)) => {
                    cur = map.get(key);
                }
                _ => return None,
            }
        }

        cur
    }

    pub fn get_id(&self) -> Option<&ObjectId> {
        match &self.id {
            Value::ObjectId(oid) => Some(oid),
            _ => None,
        }
    }

    /// Get the raw ID value (useful for testing and comparisons)
    pub fn id(&self) -> &Value {
        &self.id
    }

    pub fn ensure_id(&mut self) -> &ObjectId {
        // Check if id is already an ObjectId
        if let Value::ObjectId(ref oid) = self.id {
            return oid;
        }

        // Otherwise create and set a new ObjectId
        let new_id = ObjectId::new();
        self.id = Value::ObjectId(new_id);

        if let Value::ObjectId(ref oid) = self.id {
            oid
        } else {
            unreachable!("Document id should always be ObjectId after ensure_id");
        }
    }

    /// Get an iterator over all field names in the document
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.data.keys()
    }

    /// Get an iterator over all field values in the document
    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.data.values()
    }

    /// Get an iterator over all field name-value pairs in the document
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Value)> {
        self.data.iter()
    }

    /// Check if the document is empty (no fields)
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the number of fields in the document
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

// Returns the size of the document in bytes
fn document_size_validation(document: &str) -> bool {
    document.len() <= MAX_DOCUMENT_SIZE
}

fn document_name_validation(name: &str) -> bool {
    !name.is_empty() && name.chars().count() <= MAX_NAME_LENGTH
}

// Combine both functions for normal calls outside of this file.
pub fn validate_document(document: &str, name: &str) -> bool {
    document_size_validation(document) && document_name_validation(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::object_id::ObjectId;
    use crate::document::types::Value;

    #[test]
    fn test_new_document() {
        let doc = Document::new();
        assert!(doc.data.is_empty());
        match &doc.id {
            Value::ObjectId(_) => (),
            _ => panic!("id should be an ObjectId"),
        }
    }

    #[test]
    fn test_with_id() {
        let oid = ObjectId::new();
        let doc = Document::with_id(oid.clone());
        assert!(doc.data.is_empty());
        match &doc.id {
            Value::ObjectId(id) => assert_eq!(id, &oid),
            _ => panic!("id should be an ObjectId"),
        }
    }

    #[test]
    fn test_from_json() {
        let json = r#"{"foo": 42, "bar": true}"#;
        let doc = Document::from_json(json).unwrap();
        assert_eq!(doc.get("foo"), Some(&Value::I32(42)));
        assert_eq!(doc.get("bar"), Some(&Value::Bool(true)));
    }

    #[test]
    fn test_get_set_remove() {
        let mut doc = Document::new();
        doc.set("alpha", Value::I32(1));
        assert_eq!(doc.get("alpha"), Some(&Value::I32(1)));
        let removed = doc.remove("alpha");
        assert_eq!(removed, Some(Value::I32(1)));
        assert_eq!(doc.get("alpha"), None);
    }

    #[test]
    fn test_get_path_simple() {
        let mut doc = Document::new();
        let mut inner = std::collections::BTreeMap::new();
        inner.insert("y".to_owned(), Value::I32(9));
        doc.set("x", Value::Object(inner));
        assert_eq!(doc.get_path("x.y"), Some(&Value::I32(9)));
    }

    #[test]
    fn test_get_path_missing() {
        let doc = Document::new();
        assert_eq!(doc.get_path("no.such.path"), None);
    }

    #[test]
    fn test_get_id_and_ensure_id() {
        let mut doc = Document::new();
        // get_id should always return Some
        let id1 = doc.get_id().unwrap().clone();
        // ensure_id should return the same id
        let id2 = doc.ensure_id();
        assert_eq!(&id1, id2);
    }

    #[test]
    fn test_ensure_id_sets_id_if_missing() {
        // Manually set id to a non-ObjectId value
        let mut doc = Document::new();
        doc.id = Value::I32(123);
        let id = doc.ensure_id().clone();
        match &doc.id {
            Value::ObjectId(oid) => assert_eq!(oid, &id),
            _ => panic!("id should be ObjectId"),
        }
    }

    #[test]
    fn test_document_size_validation() {
        let valid = "a".repeat(MAX_DOCUMENT_SIZE);
        let invalid = "a".repeat(MAX_DOCUMENT_SIZE + 1);
        assert!(super::document_size_validation(&valid));
        assert!(!super::document_size_validation(&invalid));
    }

    #[test]
    fn test_document_name_validation() {
        assert!(super::document_name_validation("a_valid_name"));
        assert!(!super::document_name_validation(""));
        let long_name = "a".repeat(MAX_NAME_LENGTH + 1);
        assert!(!super::document_name_validation(&long_name));
    }

    #[test]
    fn test_validate_document() {
        let valid_doc = "a".repeat(MAX_DOCUMENT_SIZE);
        let valid_name = "goodname";
        assert!(super::validate_document(&valid_doc, &valid_name));

        let invalid_doc = "a".repeat(MAX_DOCUMENT_SIZE + 1);
        assert!(!super::validate_document(&invalid_doc, &valid_name));

        let invalid_name = "";
        assert!(!super::validate_document(&valid_doc, &invalid_name));
    }
}

// Example 1: Nested user profile with hobbies array and address object
#[allow(dead_code)]
fn example_user_profile() -> Document {
    let hobbies = vec![
        Value::String("reading".to_string()),
        Value::String("hiking".to_string()),
        Value::String("programming".to_string()),
    ];

    let mut address = BTreeMap::new();
    address.insert(
        "street".to_string(),
        Value::String("123 Main St".to_string()),
    );
    address.insert("city".to_string(), Value::String("Metropolis".to_string()));
    address.insert("zip".to_string(), Value::I32(12345));

    let mut profile = BTreeMap::new();
    profile.insert(
        "username".to_string(),
        Value::String("ethanrule".to_string()),
    );
    profile.insert("age".to_string(), Value::I32(30));
    profile.insert(
        "email".to_string(),
        Value::String("ethan@example.com".to_string()),
    );
    profile.insert("active".to_string(), Value::Bool(true));
    profile.insert("hobbies".to_string(), Value::Array(hobbies));
    profile.insert("address".to_string(), Value::Object(address));

    Document {
        data: profile,
        id: Value::ObjectId(ObjectId::new()),
    }
}

// Example 2: Document with embedded documents (e.g., posts with comments)
#[allow(dead_code)]
fn example_post_with_comments() -> Document {
    let mut comment1 = BTreeMap::new();
    comment1.insert("user".to_string(), Value::String("alice".to_string()));
    comment1.insert("text".to_string(), Value::String("Nice post!".to_string()));

    let mut comment2 = BTreeMap::new();
    comment2.insert("user".to_string(), Value::String("bob".to_string()));
    comment2.insert(
        "text".to_string(),
        Value::String("Thanks for sharing!".to_string()),
    );

    let comments = vec![Value::Object(comment1), Value::Object(comment2)];

    let mut post = BTreeMap::new();
    post.insert(
        "title".to_string(),
        Value::String("My Rust Project".to_string()),
    );
    post.insert(
        "body".to_string(),
        Value::String("Rust is awesome!".to_string()),
    );
    post.insert("likes".to_string(), Value::I32(42));
    post.insert("comments".to_string(), Value::Array(comments));

    Document {
        data: post,
        id: Value::ObjectId(ObjectId::new()),
    }
}

// Example 3: Deeply nested structure (organization/team/user)
#[allow(dead_code)]
fn example_organization_structure() -> Document {
    let mut user1 = BTreeMap::new();
    user1.insert("name".to_string(), Value::String("Charlie".to_string()));
    user1.insert("role".to_string(), Value::String("Developer".to_string()));

    let mut user2 = BTreeMap::new();
    user2.insert("name".to_string(), Value::String("Dana".to_string()));
    user2.insert("role".to_string(), Value::String("Designer".to_string()));

    let team_members = vec![Value::Object(user1), Value::Object(user2)];

    let mut team = BTreeMap::new();
    team.insert("name".to_string(), Value::String("Frontend".to_string()));
    team.insert("members".to_string(), Value::Array(team_members));

    let mut org = BTreeMap::new();
    org.insert(
        "org_name".to_string(),
        Value::String("Acme Corp".to_string()),
    );
    org.insert("teams".to_string(), Value::Array(vec![Value::Object(team)]));

    Document {
        data: org,
        id: Value::ObjectId(ObjectId::new()),
    }
}

// Example 4: Document mixing types (nulls, numbers, booleans, arrays, objects)
#[allow(dead_code)]
fn example_mixed_types() -> Document {
    use crate::document::types::Value;
    let mut doc = BTreeMap::new();
    doc.insert("null_field".to_string(), Value::Null);
    doc.insert("int_field".to_string(), Value::I32(-99));
    doc.insert("float_field".to_string(), Value::F64(std::f64::consts::PI));
    doc.insert("bool_field".to_string(), Value::Bool(false));
    doc.insert(
        "string_field".to_string(),
        Value::String("hello".to_string()),
    );
    doc.insert(
        "array_field".to_string(),
        Value::Array(vec![
            Value::I32(1),
            Value::String("two".to_string()),
            Value::Bool(true),
        ]),
    );
    doc.insert(
        "object_field".to_string(),
        Value::Object({
            let mut inner = BTreeMap::new();
            inner.insert("x".to_string(), Value::I32(123));
            inner.insert("y".to_string(), Value::String("deep".to_string()));
            inner
        }),
    );

    Document {
        data: doc,
        id: Value::ObjectId(ObjectId::new()),
    }
}
