# Rust Database Engine
Document-oriented database engine, featuring BSON document storage with page-based persistence, LRU Cache, and buffer pool management.

## Features

#### **BSON Document Storage**

- **BSON Datatypes**: Strings, Numbers (I32, I64, F64), Booleans, Arrays, Objects, ObjectIds, Null, Binary, DateTime.  
- **Validation & Safety**: Document size limits (16MB), nesting depth limits, UTF-8 validation.  
- **Performance optimized**: Partial document reading, progress callbacks, buffer reuse.  

#### **Paging Storage**

- **8KB pages** with slot directory management.  
- **Page headers** with checksums for data integrity.  
- **Slot reuse** and page compaction for space efficiency.  
- **Memory alignment** fixes for safe pointer operations.  
- **Page types**: Data, Index, and Metadata pages.  

#### **Buffer Pool Management**

- **LRU eviction** policy for memory-efficient caching.  
- **Page pinning/unpinning** for safe concurrent access.  
- **Dirty page tracking** for write-through persistence.  
- **Configurable pool size** for performance tuning.  

#### **Database File Management**

- **Database file creation** with versioning and metadata.  
- **Exclusive file locking** to prevent corruption.  
- **Header validation** and compatibility checking.  
- **Atomic operations** with proper sync/flush.  

#### **Document API**

- **Full document manipulation**: Create, set, get, and remove fields.  
- **Nested objects and arrays** with BTreeMap backing.  
- **Type-safe value system** with proper conversions.  
- **Path-based field access** for nested data.  
- **Document validation** with comprehensive error handling.  

## **BSON Format Overview**

Every document is stored as BSON (Binary JSON) with the following layout:

```
[Document Length (4 bytes)][Document Fields...][Null Terminator (1 byte)]
```

#### **[Document Fields...] Structure**

Each field follows this pattern:

```
[Type (1 byte)][Field Name (null-terminated string)][Value (variable length)]
```

#### **Supported Data Types**

| Type     | BSON Code | Rust Type                 | Storage Size         |
| -------- | --------- | ------------------------- | -------------------- |
| Double   | 0x01      | `f64`                     | 8 bytes              |
| String   | 0x02      | `String`                  | 4 + length + 1 bytes |
| Object   | 0x03      | `BTreeMap<String, Value>` | Variable             |
| Array    | 0x04      | `Vec<Value>`              | Variable             |
| Binary   | 0x05      | `Vec<u8>`                 | 4 + length bytes     |
| ObjectId | 0x07      | `ObjectId`                | 12 bytes             |
| Boolean  | 0x08      | `bool`                    | 1 byte               |
| DateTime | 0x09      | `i64` (timestamp)         | 8 bytes              |
| Null     | 0x0A      | `None`                    | 0 bytes              |
| Int32    | 0x10      | `i32`                     | 4 bytes              |
| Int64    | 0x12      | `i64`                     | 8 bytes              |

#### **Example Document Storage**

**JSON Document:**

```json
{
  "name": "Alice",
  "age": 28,
  "active": true,
  "balance": 1250.75
}
```

**BSON Binary Layout:**

```
[2F 00 00 00]           // Document length: 47 bytes
[02] [6E 61 6D 65 00]   // String "name"
[06 00 00 00] [41 6C 69 63 65 00]  // "Alice" (6 bytes including null)
[10] [61 67 65 00]      // Int32 "age"
[1C 00 00 00]           // Value: 28
[08] [61 63 74 69 76 65 00]  // Boolean "active"
[01]                    // Value: true
[01] [62 61 6C 61 6E 63 65 00]  // Double "balance"
[00 00 00 00 00 84 93 40]  // Value: 1250.75 (IEEE 754)
[00]                    // Document terminator
```

#### **Page Storage Layout**

Each 8KB page contains:  

```
[Page Header (16 bytes)][Slot Directory][Document Data]
```

**Page Header:**

- Page ID (8 bytes)  
- Checksum (4 bytes)  
- Free space counter (2 bytes)  
- Page type (1 byte)  
- Reserved (1 byte)  

**Slot Directory:**

- Array of (offset, length) pairs  
- Enables efficient document location  
- Supports tombstones for deleted documents  

## **Performance Characteristics**

#### **BSON Serialization Benchmarks**

| Document Size           | Serialization | Deserialization |
| ----------------------- | ------------- | --------------- |
| Small (10 fields)       | ~4 µs         | ~10 µs          |
| Medium (100 fields)     | ~35 µs        | ~130 µs         |
| Large (1000 fields)     | ~370 µs       | ~1.7 ms         |
| Very Large (10K fields) | ~6 ms         | ~21 ms          |

#### **Memory Usage**

- **Document size limit**: 16MB per document  
- **Page size**: 8KB (configurable)  
- **Buffer pool**: Configurable (default: 64 pages = 512KB)  
- **Memory efficiency**: Streaming operations minimize allocations  

#### **Storage Efficiency**

- **Page utilization**: Slot directory enables high space efficiency  
- **Compaction**: Automatic reclamation of deleted document space  
- **Alignment**: Proper memory alignment for performance and safety  

## **Layer Architecture**

```
Application Layer
    ↓
Document API (Document, Value types)
    ↓
BSON Serialization (Binary format)
    ↓
Storage Engine (CRUD operations)
    ↓
Buffer Pool (Memory management & LRU caching)
    ↓
Page Layout (Slot directories)
    ↓
Database File (Persistence & Disk I/O)
```

#### **Key Components**

1. **Document System** (`src/document/`)

   - `Document`: Main document structure with BTreeMap backing  
   - `Value`: Enum for all supported data types  
   - `ObjectId`: Unique 12-byte identifiers  
   - `Validator`: Document validation and constraints  

2. **BSON Engine** (`src/document/bson.rs`)

   - Streaming serialization/deserialization  
   - All BSON types supported  
   - Memory-efficient with progress tracking  

3. **Storage Engine** (`src/storage/`)
   - `StorageEngine`: High-level CRUD interface  
   - `BufferPool`: LRU cache with page management  
   - `Page`: 8KB page structure with headers  
   - `PageLayout`: Slot directory management  
   - `DatabaseFile`: File I/O and locking  

## **Testing & Quality**

#### **Test Coverage: 247 Tests Passing**

- **Unit tests**: 180 tests covering all components  
- **Integration tests**: 67 tests for end-to-end workflows  
- **Property tests**: Fuzzing and edge case validation
- **Performance tests**: Benchmarks and stress testing
