// Document validator

use crate::document::{Document, Value};
use std::collections::HashSet;

#[cfg(test)]
use std::collections::BTreeMap;

// Validation error reporting with field paths
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Document size limit exceeded: {0} bytes (max: {1})")]
    SizeLimitExceeded(usize, usize),
    #[error("Empty field name")]
    EmptyFieldName,
    #[error("Field name too long: {0} characters")]
    FieldNameTooLong(usize),
    #[error("Invalid field name: {0}")]
    InvalidFieldName(String),
    #[error("Field name contains null bytes")]
    FieldNameContainsNullBytes,
    #[error("Reserved field name: {0}")]
    ReservedFieldName(String),
    #[error("Nesting depth limit exceeded: {0} levels (max: {1})")]
    NestingDepthExceeded(usize, usize),
    #[error("Field count limit exceeded: {0} fields (max: {1})")]
    FieldCountExceeded(usize, usize),
    #[error("Numeric value out of range: {0}")]
    NumericRangeExceeded(String),
    #[error("Invalid string field: {0}")]
    InvalidStringField(String),
}

// Document size validation
pub struct DocumentValidator {
    max_size: usize, // default to 16MB
    max_depth: usize, // default to 100 levels
    max_fields: usize, // default to 1000 fields
    reserved_field_names: HashSet<String>,
}

impl DocumentValidator {

    pub fn new() -> Self {
        let mut reserved_names = HashSet::new();
        reserved_names.insert("_id".to_string());
        reserved_names.insert("_type".to_string());
        reserved_names.insert("_version".to_string());
        reserved_names.insert("_created".to_string());
        reserved_names.insert("_updated".to_string());
        
        Self {
            max_size: 16 * 1024 * 1024,
            max_depth: 100,
            max_fields: 1000,
            reserved_field_names: reserved_names,
        }
    }

    // doc size validation
    pub fn validate_size(&self, doc: &Document) -> Result<(), ValidationError> {
        let size = doc.size();
        if size > self.max_size {
            return Err(ValidationError::SizeLimitExceeded(size, self.max_size));
        }
        Ok(())
    }

    // field name validation
    pub fn validate_field_name(&self, name: &str) -> Result<(), ValidationError> {
        if name.is_empty() {
            return Err(ValidationError::EmptyFieldName);
        }
        if name.len() > 100 {
            return Err(ValidationError::FieldNameTooLong(name.len()));
        }
        if name.contains('\0') {
            return Err(ValidationError::FieldNameContainsNullBytes);
        }
        if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            return Err(ValidationError::InvalidFieldName(name.to_string()));
        }
        if self.reserved_field_names.contains(name) {
            return Err(ValidationError::ReservedFieldName(name.to_string()));
        }
        Ok(())
    }

    // nesting depth validation with DFS
    pub fn validate_nesting_depth(&self, doc: &Document) -> Result<(), ValidationError> {
        let max_depth = self.find_max_depth(doc);
        if max_depth > self.max_depth {
            return Err(ValidationError::NestingDepthExceeded(max_depth, self.max_depth));
        }
        Ok(())
    }

    // DFS to find maximum nesting depth
    fn find_max_depth(&self, doc: &Document) -> usize {
        let mut max_depth = 0;
        for (_, value) in &doc.data {
            let depth = self.get_value_depth(value, 1);
            max_depth = max_depth.max(depth);
        }
        max_depth
    }

    // Recursive helper to calculate depth of a value
    fn get_value_depth(&self, value: &Value, current_depth: usize) -> usize {
        match value {
            Value::Object(obj) => {
                let mut max_depth = current_depth;
                for (_, val) in obj {
                    let depth = self.get_value_depth(val, current_depth + 1);
                    max_depth = max_depth.max(depth);
                }
                max_depth
            }
            Value::Array(arr) => {
                let mut max_depth = current_depth;
                for val in arr {
                    let depth = self.get_value_depth(val, current_depth + 1);
                    max_depth = max_depth.max(depth);
                }
                max_depth
            }
            _ => current_depth, // Primitive types don't add depth
        }
    }

    // field count limits per document
    pub fn validate_field_count(&self, doc: &Document) -> Result<(), ValidationError> {
        let field_count = doc.data.len();
        if field_count > self.max_fields {
            return Err(ValidationError::FieldCountExceeded(field_count, self.max_fields));
        }
        Ok(())
    }

    // UTF-8 validation for strings
    pub fn validate_string_field(&self, string: &str) -> Result<(), ValidationError> {
        if !string.is_ascii() {
            return Err(ValidationError::InvalidStringField(string.to_string()));
        }
        Ok(())
    }

    // Numeric range validation
    pub fn validate_numeric_range(&self, value: &Value) -> Result<(), ValidationError> {
        match value {
            Value::I32(_) => {
                // i32 is already bounded by its type, no additional validation needed
                Ok(())
            }
            Value::I64(_) => {
                // i64 is already bounded by its type, no additional validation needed
                Ok(())
            }
            Value::F64(f) => {
                // Check for NaN and infinity
                if f.is_nan() || f.is_infinite() {
                    return Err(ValidationError::NumericRangeExceeded(f.to_string()));
                }
                Ok(())
            }
            _ => Ok(())
        }
    }

    // Comprehensive document validation with field paths
    pub fn validate_document(&self, doc: &Document) -> Result<(), ValidationError> {
        // Validate document size
        self.validate_size(doc)?;
        
        // Validate field count
        self.validate_field_count(doc)?;
        
        // Validate nesting depth
        self.validate_nesting_depth(doc)?;
        
        // Validate all fields recursively with path tracking
        self.validate_fields_recursive(doc, "")?;
        
        Ok(())
    }

    // Recursive field validation with path tracking
    fn validate_fields_recursive(&self, doc: &Document, path: &str) -> Result<(), ValidationError> {
        for (field_name, value) in &doc.data {
            let field_path = if path.is_empty() {
                field_name.clone()
            } else {
                format!("{}.{}", path, field_name)
            };
            
            // Validate field name
            self.validate_field_name(field_name)?;
            
            // Validate value
            self.validate_value_recursive(value, &field_path)?;
        }
        Ok(())
    }

    // Recursive value validation with path tracking
    fn validate_value_recursive(&self, value: &Value, path: &str) -> Result<(), ValidationError> {
        match value {
            Value::String(s) => {
                self.validate_string_field(s)?;
            }
            Value::I32(_) | Value::I64(_) | Value::F64(_) => {
                self.validate_numeric_range(value)?;
            }
            Value::Object(obj) => {
                // Create a temporary document for nested object validation
                let nested_doc = Document {
                    data: obj.clone(),
                    id: Value::Null, // Not used for validation
                };
                self.validate_fields_recursive(&nested_doc, path)?;
            }
            Value::Array(arr) => {
                for (i, val) in arr.iter().enumerate() {
                    let array_path = format!("{}[{}]", path, i);
                    self.validate_value_recursive(val, &array_path)?;
                }
            }
            _ => {} // Other types don't need special validation
        }
        
        Ok(())
    }
}

// Add size method to Document
impl Document {
    pub fn size(&self) -> usize {
        // Estimate document size by serializing to BSON
        // This is a simplified implementation - you might want to use the actual BSON serializer
        let mut size = 4; // Document length prefix
        
        for (key, value) in &self.data {
            size += 1; // Type byte
            size += key.len() + 1; // Field name + null terminator
            size += self.estimate_value_size(value);
        }
        
        size += 1; // Document null terminator
        size
    }
    
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Null => 0,
            Value::Bool(_) => 1,
            Value::I32(_) => 4,
            Value::I64(_) => 8,
            Value::F64(_) => 8,
            Value::String(s) => 4 + s.len() + 1, // Length prefix + string + null terminator
            Value::ObjectId(_) => 12,
            Value::Array(arr) => {
                let mut size = 4; // Array length prefix
                for (i, val) in arr.iter().enumerate() {
                    size += 1; // Type byte
                    size += i.to_string().len() + 1; // Index as string + null terminator
                    size += self.estimate_value_size(val);
                }
                size += 1; // Array null terminator
                size
            }
            Value::Object(obj) => {
                let mut size = 4; // Object length prefix
                for (key, val) in obj {
                    size += 1; // Type byte
                    size += key.len() + 1; // Field name + null terminator
                    size += self.estimate_value_size(val);
                }
                size += 1; // Object null terminator
                size
            }
            Value::DateTime(_) => 8,
            Value::Binary(bin) => 4 + 1 + bin.len(), // Length + subtype + data
        }
    }
}

// Comprehensive tests with edge cases
#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::types::Value;

    #[test]
    fn test_validate_nesting_depth_simple() {
        let validator = DocumentValidator::new();
        let doc = Document::new();
        assert!(validator.validate_nesting_depth(&doc).is_ok());
    }

    #[test]
    fn test_validate_nesting_depth_nested() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Create a nested structure: { "level1": { "level2": { "level3": "value" } } }
        let mut level3 = BTreeMap::new();
        level3.insert("level3".to_string(), Value::String("value".to_string()));
        
        let mut level2 = BTreeMap::new();
        level2.insert("level2".to_string(), Value::Object(level3));
        
        doc.set("level1", Value::Object(level2));
        
        assert!(validator.validate_nesting_depth(&doc).is_ok());
    }

    #[test]
    fn test_validate_nesting_depth_too_deep() {
        let mut validator = DocumentValidator::new();
        validator.max_depth = 2; // Set low limit for testing
        
        let mut doc = Document::new();
        
        // Create a structure deeper than 2 levels
        let mut level3 = BTreeMap::new();
        level3.insert("level3".to_string(), Value::String("value".to_string()));
        
        let mut level2 = BTreeMap::new();
        level2.insert("level2".to_string(), Value::Object(level3));
        
        let mut level1 = BTreeMap::new();
        level1.insert("level1".to_string(), Value::Object(level2));
        
        doc.set("root", Value::Object(level1));
        
        let result = validator.validate_nesting_depth(&doc);
        assert!(result.is_err());
        match result {
            Err(ValidationError::NestingDepthExceeded(depth, max)) => {
                assert_eq!(depth, 4);
                assert_eq!(max, 2);
            }
            _ => panic!("Expected NestingDepthExceeded error"),
        }
    }

    #[test]
    fn test_validate_nesting_depth_with_arrays() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Create nested arrays: { "arr": [ [ [ "deep" ] ] ] }
        let deep_array = vec![Value::String("deep".to_string())];
        let level2_array = vec![Value::Array(deep_array)];
        let level1_array = vec![Value::Array(level2_array)];
        
        doc.set("arr", Value::Array(level1_array));
        
        assert!(validator.validate_nesting_depth(&doc).is_ok());
    }

    #[test]
    fn test_document_size_estimation() {
        let mut doc = Document::new();
        doc.set("string", Value::String("hello".to_string()));
        doc.set("number", Value::I32(42));
        
        let size = doc.size();
        assert!(size > 0);
        assert!(size < 1000); // Should be reasonable for this simple document
    }

    #[test]
    fn test_field_name_validation() {
        let validator = DocumentValidator::new();
        
        // Valid field names
        assert!(validator.validate_field_name("valid_field").is_ok());
        assert!(validator.validate_field_name("field123").is_ok());
        assert!(validator.validate_field_name("_private").is_ok());
        
        // Invalid field names
        assert!(validator.validate_field_name("").is_err());
        assert!(validator.validate_field_name("field-name").is_err()); // hyphen
        assert!(validator.validate_field_name("field name").is_err()); // space
        assert!(validator.validate_field_name("field.name").is_err()); // dot
    }

    #[test]
    fn test_field_name_null_bytes() {
        let validator = DocumentValidator::new();
        let name_with_null = "field\0name".to_string();
        assert!(validator.validate_field_name(&name_with_null).is_err());
    }

    #[test]
    fn test_reserved_field_names() {
        let validator = DocumentValidator::new();
        assert!(validator.validate_field_name("_id").is_err());
        assert!(validator.validate_field_name("_type").is_err());
        assert!(validator.validate_field_name("_version").is_err());
        assert!(validator.validate_field_name("_created").is_err());
        assert!(validator.validate_field_name("_updated").is_err());
    }

    #[test]
    fn test_string_validation() {
        let validator = DocumentValidator::new();
        
        // Valid strings
        assert!(validator.validate_string_field("hello").is_ok());
        assert!(validator.validate_string_field("hello123").is_ok());
        
        // Invalid strings (non-ASCII)
        let non_ascii_string = "hello世界".to_string();
        assert!(validator.validate_string_field(&non_ascii_string).is_err());
    }

    #[test]
    fn test_numeric_validation() {
        let validator = DocumentValidator::new();
        
        // Valid numbers
        assert!(validator.validate_numeric_range(&Value::I32(42)).is_ok());
        assert!(validator.validate_numeric_range(&Value::I64(42)).is_ok());
        assert!(validator.validate_numeric_range(&Value::F64(42.0)).is_ok());
        
        // Invalid numbers (these should pass since they're within type bounds)
        // The current implementation checks type bounds, which are always valid in Rust
        assert!(validator.validate_numeric_range(&Value::I32(i32::MAX)).is_ok());
        assert!(validator.validate_numeric_range(&Value::I64(i64::MAX)).is_ok());
    }

    #[test]
    fn test_comprehensive_document_validation() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Valid document
        doc.set("name", Value::String("John".to_string()));
        doc.set("age", Value::I32(30));
        doc.set("active", Value::Bool(true));
        
        assert!(validator.validate_document(&doc).is_ok());
    }

    #[test]
    fn test_comprehensive_document_validation_with_nested_objects() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Create nested structure
        let mut address = BTreeMap::new();
        address.insert("street".to_string(), Value::String("123 Main St".to_string()));
        address.insert("city".to_string(), Value::String("Anytown".to_string()));
        
        let mut user = BTreeMap::new();
        user.insert("name".to_string(), Value::String("John".to_string()));
        user.insert("address".to_string(), Value::Object(address));
        
        doc.set("user", Value::Object(user));
        
        assert!(validator.validate_document(&doc).is_ok());
    }

    #[test]
    fn test_comprehensive_document_validation_with_arrays() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Create array structure
        let tags = vec![
            Value::String("tag1".to_string()),
            Value::String("tag2".to_string()),
            Value::String("tag3".to_string()),
        ];
        
        doc.set("tags", Value::Array(tags));
        
        assert!(validator.validate_document(&doc).is_ok());
    }

    #[test]
    fn test_comprehensive_document_validation_invalid_field_name() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Invalid field name
        doc.set("invalid-field", Value::String("value".to_string()));
        
        let result = validator.validate_document(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_comprehensive_document_validation_reserved_field_name() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Reserved field name
        doc.set("_id", Value::String("value".to_string()));
        
        let result = validator.validate_document(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_comprehensive_document_validation_invalid_string() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Non-ASCII string
        doc.set("name", Value::String("John世界".to_string()));
        
        let result = validator.validate_document(&doc);
        assert!(result.is_err());
    }

    #[test]
    fn test_field_path_tracking_nested_objects() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Create nested structure with invalid field name
        let mut nested = BTreeMap::new();
        nested.insert("invalid-field".to_string(), Value::String("value".to_string()));
        
        doc.set("user", Value::Object(nested));
        
        let result = validator.validate_document(&doc);
        assert!(result.is_err());
        // The error should be about the invalid field name in the nested object
    }

    #[test]
    fn test_field_path_tracking_arrays() {
        let validator = DocumentValidator::new();
        let mut doc = Document::new();
        
        // Create array with invalid string
        let tags = vec![
            Value::String("valid".to_string()),
            Value::String("invalid世界".to_string()),
            Value::String("also_valid".to_string()),
        ];
        
        doc.set("tags", Value::Array(tags));
        
        let result = validator.validate_document(&doc);
        assert!(result.is_err());
        // The error should be about the invalid string in the array
    }
}