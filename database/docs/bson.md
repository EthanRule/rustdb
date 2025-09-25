# BSON Format Documentation

## Overview

BSON (Binary JSON) is a binary serialization format used by MongoDB and this database engine. It stores documents in a compact binary format with type information and length prefixes.

## Document Structure

### Document Header
Every BSON document starts with a 4-byte little-endian length prefix:

```
[Document Length (4 bytes)][Document Fields...][Null Terminator (1 byte)]
```

### Field Structure
Each field in a document follows this pattern:

```
[Type (1 byte)][Field Name (null-terminated string)][Value (variable length)]
```

## Supported Types

| Type | Value | Description |
|------|-------|-------------|
| 0x01 | Double | 8-byte IEEE 754 double |
| 0x02 | String | Length-prefixed UTF-8 string |
| 0x03 | Object | Embedded document |
| 0x04 | Array | Array (stored as object with numeric keys) |
| 0x05 | Binary | Length-prefixed binary data |
| 0x07 | ObjectId | 12-byte ObjectId |
| 0x08 | Boolean | 1-byte boolean (0x00 = false, 0x01 = true) |
| 0x09 | DateTime | 8-byte UTC timestamp (milliseconds since epoch) |
| 0x0A | Null | No additional data |
| 0x10 | Int32 | 4-byte little-endian integer |
| 0x12 | Int64 | 8-byte little-endian integer |

## Byte Layout Examples

### Example 1: Simple Document
```json
{
  "name": "John",
  "age": 30,
  "active": true
}
```

**BSON Layout:**
```
[2D 00 00 00]     // Document length: 45 bytes (little-endian)
[02]              // Type: String
[6E 61 6D 65 00]  // Field name: "name" + null terminator
[05 00 00 00]     // String length: 5 bytes (including null)
[4A 6F 68 6E 00]  // String value: "John" + null terminator
[10]              // Type: Int32
[61 67 65 00]     // Field name: "age" + null terminator
[1E 00 00 00]     // Int32 value: 30 (little-endian)
[08]              // Type: Boolean
[61 63 74 69 76 65 00]  // Field name: "active" + null terminator
[01]              // Boolean value: true
[00]              // Document null terminator
```

**Total: 45 bytes**

### Example 2: Document with Nested Object
```json
{
  "user": {
    "name": "Alice",
    "email": "alice@example.com"
  },
  "score": 95.5
}
```

**BSON Layout:**
```
[4A 00 00 00]     // Document length: 74 bytes
[03]              // Type: Object
[75 73 65 72 00]  // Field name: "user" + null terminator
[2A 00 00 00]     // Embedded document length: 42 bytes
[02]              // Type: String
[6E 61 6D 65 00]  // Field name: "name" + null terminator
[06 00 00 00]     // String length: 6 bytes
[41 6C 69 63 65 00]  // String value: "Alice" + null terminator
[02]              // Type: String
[65 6D 61 69 6C 00]  // Field name: "email" + null terminator
[11 00 00 00]     // String length: 17 bytes
[61 6C 69 63 65 40 65 78 61 6D 70 6C 65 2E 63 6F 6D 00]  // "alice@example.com" + null
[00]              // Embedded document null terminator
[01]              // Type: Double
[73 63 6F 72 65 00]  // Field name: "score" + null terminator
[00 00 00 00 00 40 57 40]  // Double value: 95.5 (IEEE 754)
[00]              // Document null terminator
```

### Example 3: Array
```json
{
  "tags": ["rust", "database", "bson"]
}
```

**BSON Layout:**
```
[3A 00 00 00]     // Document length: 58 bytes
[04]              // Type: Array
[74 61 67 73 00]  // Field name: "tags" + null terminator
[2A 00 00 00]     // Array document length: 42 bytes
[02]              // Type: String
[30 00]           // Field name: "0" + null terminator
[05 00 00 00]     // String length: 5 bytes
[72 75 73 74 00]  // String value: "rust" + null terminator
[02]              // Type: String
[31 00]           // Field name: "1" + null terminator
[09 00 00 00]     // String length: 9 bytes
[64 61 74 61 62 61 73 65 00]  // String value: "database" + null terminator
[02]              // Type: String
[32 00]           // Field name: "2" + null terminator
[04 00 00 00]     // String length: 4 bytes
[62 73 6F 6E 00]  // String value: "bson" + null terminator
[00]              // Array document null terminator
[00]              // Document null terminator
```

### Example 4: All Data Types
```json
{
  "null_field": null,
  "bool_field": true,
  "int32_field": 42,
  "int64_field": 123456789,
  "double_field": 3.14159,
  "string_field": "hello",
  "objectid_field": "507f1f77bcf86cd799439011",
  "datetime_field": "2023-01-01T12:00:00Z",
  "binary_field": "AQIDBA=="
}
```

**BSON Layout:**
```
[8F 00 00 00]     // Document length: 143 bytes
[0A]              // Type: Null
[6E 75 6C 6C 5F 66 69 65 6C 64 00]  // Field name: "null_field" + null
[08]              // Type: Boolean
[62 6F 6F 6C 5F 66 69 65 6C 64 00]  // Field name: "bool_field" + null
[01]              // Boolean value: true
[10]              // Type: Int32
[69 6E 74 33 32 5F 66 69 65 6C 64 00]  // Field name: "int32_field" + null
[2A 00 00 00]     // Int32 value: 42
[12]              // Type: Int64
[69 6E 74 36 34 5F 66 69 65 6C 64 00]  // Field name: "int64_field" + null
[15 CD 5B 07 00 00 00 00]  // Int64 value: 123456789
[01]              // Type: Double
[64 6F 75 62 6C 65 5F 66 69 65 6C 64 00]  // Field name: "double_field" + null
[6E 86 1B F0 F9 21 09 40]  // Double value: 3.14159
[02]              // Type: String
[73 74 72 69 6E 67 5F 66 69 65 6C 64 00]  // Field name: "string_field" + null
[06 00 00 00]     // String length: 6 bytes
[68 65 6C 6C 6F 00]  // String value: "hello" + null
[07]              // Type: ObjectId
[6F 62 6A 65 63 74 69 64 5F 66 69 65 6C 64 00]  // Field name: "objectid_field" + null
[50 7F 1F 77 BC F8 6C D7 99 43 90 11]  // ObjectId: 12 bytes
[09]              // Type: DateTime
[64 61 74 65 74 69 6D 65 5F 66 69 65 6C 64 00]  // Field name: "datetime_field" + null
[00 00 00 00 00 00 00 00]  // DateTime: 0 (epoch)
[05]              // Type: Binary
[62 69 6E 61 72 79 5F 66 69 65 6C 64 00]  // Field name: "binary_field" + null
[04 00 00 00]     // Binary length: 4 bytes
[00]              // Binary subtype: 0 (generic)
[01 02 03 04]     // Binary data
[00]              // Document null terminator
```

## Error Handling

The BSON implementation includes comprehensive error handling for:

- **InvalidLength**: Document length doesn't match actual data size
- **UnexpectedEndOfData**: Attempting to read beyond available data
- **InvalidType**: Unknown BSON type encountered
- **InvalidString**: Invalid UTF-8 encoding in strings
- **InvalidStringLength**: Negative or zero string length
- **InvalidBinaryLength**: Negative binary length
- **InvalidTimestamp**: Invalid DateTime timestamp
- **MalformedFieldName**: Empty or malformed field names
- **MissingNullTerminator**: Missing null terminator in strings
- **DocumentTooLarge**: Document exceeds 16MB limit
- **InvalidEmbeddedDocument**: Malformed nested documents

## Implementation Notes

### Length Prefixing
- All documents are prefixed with a 4-byte little-endian length
- The length includes the 4-byte length field itself
- Maximum document size is 16MB (16,777,216 bytes)

### String Encoding
- Strings are stored as length-prefixed UTF-8
- Length includes the null terminator
- Field names are null-terminated strings

### Array Storage
- Arrays are stored as objects with numeric string keys ("0", "1", "2", etc.)
- Keys are converted to array indices during deserialization

### Binary Data
- Binary data includes a subtype byte (0 for generic binary)
- Length is stored as a 4-byte little-endian integer

### DateTime
- DateTime values are stored as 8-byte little-endian timestamps
- Timestamps represent milliseconds since Unix epoch (January 1, 1970 UTC)

### ObjectId
- ObjectId values are stored as 12-byte binary data
- No length prefix or null terminator

## Usage Examples

```rust
use crate::document::{Document, Value};
use crate::document::bson::{serialize_document, deserialize_document};

// Create a document
let mut doc = Document::new();
doc.set("name", Value::String("John".to_string()));
doc.set("age", Value::I32(30));

// Serialize to BSON
let bson_data = serialize_document(&doc)?;

// Deserialize from BSON
let decoded_doc = deserialize_document(&bson_data)?;

// Round-trip should be equal
assert_eq!(doc.data, decoded_doc.data);
```

## Performance Considerations

- **Memory Efficiency**: BSON is more compact than JSON for binary data
- **Parsing Speed**: Binary format is faster to parse than text-based formats
- **Type Safety**: Built-in type information eliminates parsing ambiguity
- **Streaming**: Length prefixes enable efficient streaming and random access

## Compatibility

This BSON implementation is compatible with MongoDB's BSON format for the supported types. However, some advanced MongoDB-specific types (like Decimal128, MinKey, MaxKey) are not currently supported. 