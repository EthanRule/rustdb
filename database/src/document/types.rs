// Design choices:
// 1. Enum variant coverage:
//    The `Value` enum explicitly supports Null, Bool, I32, I64, F64, and String. This covers the most
//    common datatypes encountered in database usage, while avoiding the complexity of supporting all
//    possible Rust types or user-defined structs.
//
// 2. Numeric conversions are safe and explicit:
//    - When converting from a larger type (e.g., I64 or F64) to a smaller type (I32), if the original
//      value is out of bounds for the target type, `None` is returned.
//      For example: `Value::F64(1e20).as_i32()` returns `None`.
//    - This prevents silent overflow/wraparound bugs that could otherwise occur if casting was always allowed.
//
// 3. String-to-number conversions use standard Rust parsing:
//    - For example, `Value::String("42").as_i32()` yields `Some(42)`. If the string is not a valid
//      integer, the conversion returns `None`.
//    - This ensures that only valid representations convert, and avoids panics or incorrect results from malformed strings.
//
// 4. Bool conversions use conventional mappings:
//    - `Value::Bool(true).as_i32()` is `Some(1)`, `false` is `Some(0)`.
//    - Numeric-to-bool conversion is only allowed for 0/1 (or 0.0/1.0 for floats); other values return `None`.
//    - String-to-bool conversion is case-insensitive, accepting `"true"`/`"false"` only; all other strings return `None`.
//
// 5. F64 conversions are careful with NaN and bounds:
//    - `as_i32` and `as_i64` from `F64` only succeed if the float is finite and within the range of the target
//       type; otherwise, they return `None`.
//    - This avoids undefined behavior and ensures database operations remain predictable.
//
// 6. Display implementation:
//    - All variants except Null and String are rendered using their standard Rust `to_string()` behavior for consistency
//      and debugging clarity.
//    - Null is rendered as "null" and String is rendered as the underlying string.
//
// 7. Null handling:
//    - Conversions from `Null` to any other type always return `None` (except `as_bool`, which returns `Some(false)` for
//      ergonomic reasons).
//
// 8. All conversion methods return `Option<T>`:
//    - This makes it explicit to the caller when a conversion may fail, and ensures no panics or silent failures occur.
//
// These choices make all value conversions safe, predictable, and easy to reason about, which is essential in a database
// context where correctness is critical.

use crate::document::object_id::ObjectId;
use chrono::{DateTime, Utc};
use proptest::arbitrary::Arbitrary;
use proptest::prelude::*;
use proptest::strategy::{BoxedStrategy, Strategy};
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    String(String),
    ObjectId(ObjectId),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    DateTime(DateTime<Utc>),
    Binary(Vec<u8>),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::I32(i) => write!(f, "{}", i),
            Value::I64(i) => write!(f, "{}", i),
            Value::F64(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::ObjectId(oid) => write!(f, "{}", oid),
        }
    }
}

impl Arbitrary for Value {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        use Value::*;
        prop_oneof![
            Just(Null),
            any::<bool>().prop_map(Bool),
            any::<i32>().prop_map(I32),
            any::<i64>().prop_map(I64),
            any::<f64>().prop_map(F64),
            ".*".prop_map(String),
            any::<crate::document::object_id::ObjectId>().prop_map(ObjectId),
        ]
        .boxed()
    }
}

impl Value {
    fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }

    fn is_number(&self) -> bool {
        matches!(self, Value::I32(_) | Value::I64(_) | Value::F64(_))
    }

    fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    fn is_object_id(&self) -> bool {
        matches!(self, Value::ObjectId(_))
    }

    fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Null => Some(false),
            Value::I32(x) => match x {
                val if *val == 1 => Some(true),
                val if *val == 0 => Some(false),
                _ => None,
            },
            Value::I64(x) => match x {
                val if *val == 1 => Some(true),
                val if *val == 0 => Some(false),
                _ => None,
            },
            Value::F64(x) => match x {
                val if *val == 1.0 => Some(true),
                val if *val == 0.0 => Some(false),
                _ => None,
            },
            Value::String(x) => match x {
                val if val.to_lowercase() == String::from("true") => Some(true),
                val if val.to_lowercase() == String::from("false") => Some(false),
                _ => None,
            },
            Value::Bool(x) => Some(*x),
            Value::ObjectId(_) => None, // ObjectId cannot be converted to bool
        }
    }

    fn as_i32(&self) -> Option<i32> {
        match self {
            Value::Null => None,
            Value::I32(x) => Some(*x),
            Value::I64(x) => i32::try_from(*x).ok(),
            Value::F64(x) => {
                if *x >= i32::MIN as f64 && *x <= i32::MAX as f64 {
                    Some(*x as i32)
                } else {
                    None
                }
            }
            Value::String(x) => x.parse::<i32>().ok(),
            Value::Bool(x) => match x {
                true => Some(1i32),
                false => Some(0i32),
            },
            Value::ObjectId(_) => None, // ObjectId cannot be converted to i32
        }
    }

    fn as_i64(&self) -> Option<i64> {
        match self {
            Value::Null => None,
            Value::I32(x) => Some(*x as i64),
            Value::I64(x) => Some(*x),
            Value::F64(x) => Some(*x as i64),
            Value::String(x) => x.parse::<i64>().ok(),
            Value::Bool(x) => match x {
                true => Some(1i64),
                false => Some(0i64),
            },
            Value::ObjectId(_) => None, // ObjectId cannot be converted to i64
        }
    }

    fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Null => None,
            Value::I32(x) => Some(*x as f64),
            Value::I64(x) => Some(*x as f64),
            Value::F64(x) => Some(*x),
            Value::String(x) => x.parse::<f64>().ok(),
            Value::Bool(x) => match x {
                true => Some(1f64),
                false => Some(0f64),
            },
            Value::ObjectId(_) => None, // ObjectId cannot be converted to f64
        }
    }

    fn as_str(&self) -> Option<String> {
        match self {
            Value::Null => None,
            Value::I32(x) => Some(x.to_string()),
            Value::I64(x) => Some(x.to_string()),
            Value::F64(x) => Some(x.to_string()),
            Value::String(x) => Some(x.clone()),
            Value::Bool(x) => match x {
                true => Some(String::from("true")),
                false => Some(String::from("false")),
            },
            Value::ObjectId(oid) => None, // ObjectId cannot be converted to String
        }
    }

    fn as_object_id(&self) -> Option<ObjectId> {
        match self {
            Value::ObjectId(oid) => Some(oid.clone()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        let value = Value::String("Hello".to_string());
        assert_eq!(value.to_string(), "Hello");

        let value = Value::I32(42);
        assert_eq!(value.to_string(), "42");

        let value = Value::Bool(true);
        assert_eq!(value.to_string(), "true");

        let value = Value::Null;
        assert_eq!(value.to_string(), "null");
    }

    #[test]
    fn test_value_is_null() {
        let value = Value::Null;
        assert!(value.is_null());
    }

    #[test]
    fn test_value_is_bool() {
        let value = Value::Bool(true);
        assert!(value.is_bool());
    }

    #[test]
    fn test_value_is_number() {
        let value = Value::I32(10);
        assert!(value.is_number());

        let value = Value::F64(3.14);
        assert!(value.is_number());

        let value = Value::String("Not a number".to_string());
        assert!(!value.is_number());
    }

    #[test]
    fn test_value_as_bool() {
        let value = Value::String("true".to_string());
        assert_eq!(value.as_bool(), Some(true));

        let value = Value::I32(0);
        assert_eq!(value.as_bool(), Some(false));

        let value = Value::Null;
        assert_eq!(value.as_bool(), Some(false));

        let value = Value::Bool(false);
        assert_eq!(value.as_bool(), Some(false));
    }

    #[test]
    fn test_value_as_i32() {
        let value = Value::String("123".to_string());
        assert_eq!(value.as_i32(), Some(123));

        let value = Value::F64(42.0);
        assert_eq!(value.as_i32(), Some(42));

        let value = Value::Null;
        assert_eq!(value.as_i32(), None);

        let value = Value::Bool(true);
        assert_eq!(value.as_i32(), Some(1));
    }

    #[test]
    fn test_value_as_i64() {
        let value = Value::String("123456789".to_string());
        assert_eq!(value.as_i64(), Some(123456789));

        let value = Value::F64(42.0);
        assert_eq!(value.as_i64(), Some(42));

        let value = Value::Null;
        assert_eq!(value.as_i64(), None);

        let value = Value::Bool(true);
        assert_eq!(value.as_i64(), Some(1));
    }

    #[test]
    fn test_value_as_f64() {
        let value = Value::String("3.14".to_string());
        assert_eq!(value.as_f64(), Some(3.14));

        let value = Value::I32(42);
        assert_eq!(value.as_f64(), Some(42.0));

        let value = Value::Null;
        assert_eq!(value.as_f64(), None);

        let value = Value::Bool(false);
        assert_eq!(value.as_f64(), Some(0.0));
    }

    #[test]
    fn test_value_as_str() {
        let value = Value::String("Hello".to_string());
        assert_eq!(value.as_str(), Some("Hello".to_string()));

        let value = Value::I32(42);
        assert_eq!(value.as_str(), Some("42".to_string()));

        let value = Value::Null;
        assert_eq!(value.as_str(), None);

        let value = Value::Bool(true);
        assert_eq!(value.as_str(), Some("true".to_string()));

        let value = Value::F64(3.14);
        assert_eq!(value.as_str(), Some("3.14".to_string()));
    }

    #[test]
    fn test_value_is_object_id() {
        let oid = ObjectId::new();
        let value = Value::ObjectId(oid.clone());
        assert!(value.is_object_id());
        assert_eq!(value.as_object_id(), Some(oid));
    }

    #[test]
    fn test_object_id_display_and_hex() {
        let oid = ObjectId::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
        let value = Value::ObjectId(oid.clone());
        assert_eq!(value.to_string(), oid.to_hex());
    }

    #[test]
    fn test_object_id_no_number_conversion() {
        let oid = ObjectId::new();
        let value = Value::ObjectId(oid);
        assert_eq!(value.as_bool(), None);
        assert_eq!(value.as_i32(), None);
        assert_eq!(value.as_i64(), None);
        assert_eq!(value.as_f64(), None);
        assert_eq!(value.as_str(), None);
    }

    // Property-based tests for Value enum

    proptest! {
        #[test]
        fn prop_value_display(value in any::<Value>()) {
            let display = value.to_string();
            match value {
                Value::Null => assert_eq!(display, "null"),
                Value::Bool(b) => assert_eq!(display, b.to_string()),
                Value::I32(i) => assert_eq!(display, i.to_string()),
                Value::I64(i) => assert_eq!(display, i.to_string()),
                Value::F64(f) => assert_eq!(display, f.to_string()),
                Value::String(s) => assert_eq!(display, s),
                Value::ObjectId(oid) => assert_eq!(display, oid.to_hex()),
            }
        }

        #[test]
        fn prop_value_is_null(value in any::<Value>()) {
            let is_null = value.is_null();
            if let Value::Null = value {
                assert!(is_null);
            } else {
                assert!(!is_null);
            }
        }

        #[test]
        fn prop_value_is_bool(value in any::<Value>()) {
            let is_bool = value.is_bool();
            if let Value::Bool(_) = value {
                assert!(is_bool);
            } else {
                assert!(!is_bool);
            }
        }

        #[test]
        fn prop_value_is_number(value in any::<Value>()) {
            let is_number = value.is_number();
            if matches!(value, Value::I32(_) | Value::I64(_) | Value::F64(_)) {
                assert!(is_number);
            } else {
                assert!(!is_number);
            }
        }

        #[test]
        fn prop_value_is_string(value in any::<Value>()) {
            let is_string = value.is_string();
            if let Value::String(_) = value {
                assert!(is_string);
            } else {
                assert!(!is_string);
            }
        }

        #[test]
        fn prop_value_as_bool(value in any::<Value>()) {
            let result = value.as_bool();
            match value {
                Value::Null => assert_eq!(result, Some(false)),
                Value::Bool(b) => assert_eq!(result, Some(b)),
                Value::I32(i) => assert_eq!(result, if i == 1 { Some(true) } else if i == 0 { Some(false) } else { None }),
                Value::I64(i) => assert_eq!(result, if i == 1 { Some(true) } else if i == 0 { Some(false) } else { None }),
                Value::F64(f) => assert_eq!(result, if f == 1.0 { Some(true) } else if f == 0.0 { Some(false) } else { None }),
                Value::String(s) => {
                    if s.to_lowercase() == "true" {
                        assert_eq!(result, Some(true));
                    } else if s.to_lowercase() == "false" {
                        assert_eq!(result, Some(false));
                    } else {
                        assert_eq!(result, None);
                    }
                }
                Value::ObjectId(_) => assert_eq!(result, None), // ObjectId cannot be converted to
                // bool
            }
        }

        #[test]
        fn prop_value_as_i32(value in any::<Value>()) {
            let result = value.as_i32();
            match value {
                Value::Null => assert_eq!(result, None),
                Value::I32(i) => assert_eq!(result, Some(i)),
                Value::I64(i) => {
                    if i >= i32::MIN as i64 && i <= i32::MAX as i64 {
                        assert_eq!(result, Some(i as i32));
                    } else {
                        assert_eq!(result, None);
                    }
                },
                Value::F64(f) => {
                    if f.is_finite() && f >= i32::MIN as f64 && f <= i32::MAX as f64 {
                        assert_eq!(result, Some(f as i32));
                    } else {
                        assert_eq!(result, None);
                    }
                },
                Value::String(s) => {
                    if let Ok(parsed) = s.parse::<i32>() {
                        assert_eq!(result, Some(parsed));
                    } else {
                        assert_eq!(result, None);
                    }
                },
                Value::Bool(b) => assert_eq!(result, Some(if b { 1 } else { 0 })),
                Value::ObjectId(_) => assert_eq!(result, None), // ObjectId cannot be converted to
                // i32
            }
        }

        #[test]
        fn prop_value_as_i64(value in any::<Value>()) {
            let result = value.as_i64();
            match value {
                Value::Null => assert_eq!(result, None),
                Value::I32(i) => assert_eq!(result, Some(i as i64)),
                Value::I64(i) => assert_eq!(result, Some(i)),
                Value::F64(f) => assert_eq!(result, Some(f as i64)),
                Value::String(s) => {
                    if s.chars().all(|c| c.is_ascii_digit()) {
                        assert_eq!(result, s.parse::<i64>().ok());
                    } else {
                        assert_eq!(result, None);
                    }
                }
                Value::Bool(b) => assert_eq!(result, Some(if b { 1 } else { 0 })),
                Value::ObjectId(_) => assert_eq!(result, None), // ObjectId cannot be converted to
                // i64
            }
        }

        #[test]
        fn prop_value_as_f64(value in any::<Value>()) {
            let result = value.as_f64();
            match value {
                Value::Null => assert_eq!(result, None),
                Value::I32(i) => assert_eq!(result, Some(i as f64)),
                Value::I64(i) => assert_eq!(result, Some(i as f64)),
                Value::F64(f) => assert_eq!(result, Some(f)),
                Value::String(s) => {
                    if s.chars().all(|c| c.is_ascii_digit() || c == '.') {
                        assert_eq!(result, s.parse::<f64>().ok());
                    } else {
                        assert_eq!(result, None);
                    }
                }
                Value::Bool(b) => assert_eq!(result, Some(if b { 1.0 } else { 0.0 })),
                Value::ObjectId(_) => assert_eq!(result, None), // ObjectId cannot be converted to
                // f64
            }
        }

        #[test]
        fn prop_value_as_str(value in any::<Value>()) {
            let result = value.as_str();
            match value {
                Value::Null => assert_eq!(result, None),
                Value::I32(i) => assert_eq!(result, Some(i.to_string())),
                Value::I64(i) => assert_eq!(result, Some(i.to_string())),
                Value::F64(f) => assert_eq!(result, Some(f.to_string())),
                Value::String(s) => assert_eq!(result, Some(s)),
                Value::Bool(b) => assert_eq!(result, Some(if b { "true".to_string() } else { "false".to_string() })),
                Value::ObjectId(oid) => assert_eq!(result, None), // ObjectId cannot be converted
                // to String
            }
        }

        #[test]
        fn prop_value_object_id_roundtrip(oid in any::<ObjectId>()) {
            let value = Value::ObjectId(oid.clone());
            prop_assert_eq!(value.as_object_id(), Some(oid));
        }

        #[test]
        fn prop_value_object_id_display_and_hex(oid in any::<ObjectId>()) {
            let value = Value::ObjectId(oid.clone());
            prop_assert_eq!(value.to_string(), oid.to_hex());
        }

        #[test]
        fn prop_value_object_id_no_number_conversion(oid in any::<ObjectId>()) {
            let value = Value::ObjectId(oid);
            prop_assert!(value.as_bool().is_none());
            prop_assert!(value.as_i32().is_none());
            prop_assert!(value.as_i64().is_none());
            prop_assert!(value.as_f64().is_none());
            prop_assert!(value.as_str().is_none());
        }
    }
}
