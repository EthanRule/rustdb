use crate::document::object_id::ObjectId;
use crate::document::types::Value;
use crate::error::DatabaseError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const MAX_DOCUMENT_SIZE: usize = 16 * 1024 * 1024; // 16mb
const MAX_NAME_LENGTH: usize = 100; // 100 chars

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Document {
    data: BTreeMap<String, Value>,
    id: Value,
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
        let map: BTreeMap<String, Value> = serde_json::from_str(input)?;
        Ok(Document {
            data: map,
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
        let mut cur = None;

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
            _ => return None,
        }
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
