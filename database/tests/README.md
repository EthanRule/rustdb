# Test Organization

This directory contains the test suite for the RustDB database project, organized into logical categories:

## Directory Structure

### `/integration/`
Integration tests that test multiple components working together, often involving file I/O, storage engines, and full system workflows:

- `buffer_pool_integration.rs` - Tests buffer pool functionality with actual file operations
- `crud_operations_test.rs` - Tests complete CRUD (Create, Read, Update, Delete) operations
- `page_layout_integration.rs` - Tests page layout with actual page structures
- `storage_engine_extended_test.rs` - Extended tests for storage engine functionality
- `storage_engine_test.rs` - Basic storage engine integration tests
- `week1_integration.rs` - Document-level integration tests from week 1 development
- `week2_integration.rs` - Full document lifecycle integration tests from week 2 development

### `/unit/`
Unit tests that test individual functions or components in isolation:

- `bson_roundtrip_tests.rs` - Tests BSON serialization and deserialization functionality
- `simple_page_test.rs` - Tests basic page operations in isolation

### `/property/`
Property-based and fuzz tests that test system properties and edge cases:

- `document_iteration_test.rs` - Tests document iteration behavior
- `id_persistence_test.rs` - Tests ID persistence across operations
- `page_layout_advanced_qa.rs` - Advanced quality assurance tests for page layout
- `page_layout_fuzz.rs` - Fuzz testing for page layout operations
- `page_layout_qa.rs` - Quality assurance tests for page layout

### `/debug/`
Debug and development tests used for troubleshooting specific issues:

- `compaction_bug_test.rs` - Test for reproducing specific compaction bugs
- `debug_compaction.rs` - Debug tests for compaction issues
- `debug_file_locks.rs` - Debug tests for file locking issues

## Running Tests

### Run all tests:
```bash
cargo test
```

### Run tests by category:
```bash
# Integration tests
cargo test --test integration_tests

# Unit tests  
cargo test --test unit_tests

# Property tests
cargo test --test property_tests

# Debug tests (if needed)
cargo test --test debug_tests
```

### Run specific test files:
```bash
# Example: Run BSON roundtrip tests
cargo test --test unit::bson_roundtrip_tests

# Example: Run CRUD operations tests
cargo test --test integration::crud_operations_test
```

## Test Guidelines

- **Unit tests**: Should test single functions/components in isolation, avoid file I/O
- **Integration tests**: Test multiple components together, can use file system and external dependencies
- **Property tests**: Test system invariants and edge cases using property-based testing
- **Debug tests**: Temporary tests for debugging specific issues, should be removed when issues are resolved

## Adding New Tests

When adding new tests, place them in the appropriate category:

1. **Unit tests** - for testing individual functions or small components
2. **Integration tests** - for testing multiple components working together
3. **Property tests** - for property-based testing and fuzz testing
4. **Debug tests** - only for temporary debugging (should be cleaned up regularly)

Make sure to follow the existing naming conventions and include descriptive test names.