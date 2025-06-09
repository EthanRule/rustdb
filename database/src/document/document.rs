use crate::document::object_id::ObjectId;
use crate::document::types::Value;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Document {
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

    pub fn set(&mut self, input: String, val: Value) {
        self.data.insert(input, val);
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
}
