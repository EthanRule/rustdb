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

### Unit Tests
Unit tests are located **directly in the source files** using `#[cfg(test)]` modules. This is the recommended Rust practice for testing individual functions and components in isolation:

- `src/error.rs` - Tests error handling and conversions
- `src/document/mod.rs` - Tests Document API methods
- `src/document/bson.rs` - Comprehensive BSON serialization/deserialization tests
- `src/document/object_id.rs` - Tests ObjectId functionality
- `src/storage/page_layout.rs` - Tests page layout operations
- `src/storage/buffer_pool.rs` - Tests buffer pool functionality
- And more throughout the codebase...

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
- `simple_page_test.rs` - Simple page debugging test

## Running Tests

### Run all tests:
```bash
cargo test
```

### Run tests by category:
```bash
# All tests (includes unit tests in source files)
cargo test

# Integration tests only
cargo test --test integration_tests

# Property tests only
cargo test --test property_tests

# Debug tests only (if needed)
cargo test --test debug_tests

# Unit tests only (in source files)
cargo test --lib
```

### Run specific test files:
```bash
# Example: Run BSON tests (in source file)
cargo test --lib bson::tests

# Example: Run CRUD operations tests
cargo test --test integration_tests crud_operations_test

# Example: Run Document tests (in source file)  
cargo test --lib document::tests
```

## Test Guidelines

- **Unit tests** (in source files): Test single functions/components in isolation, avoid file I/O, have access to private functions
- **Integration tests** (in `tests/` directory): Test multiple components together, can use file system and external dependencies, test public API only
- **Property tests**: Test system invariants and edge cases using property-based testing
- **Debug tests**: Temporary tests for debugging specific issues, should be removed when issues are resolved

## Adding New Tests

When adding new tests, place them in the appropriate location:

1. **Unit tests** - Add `#[cfg(test)]` modules directly in source files (`src/`) for testing individual functions or small components
2. **Integration tests** - Add to `tests/integration/` for testing multiple components working together  
3. **Property tests** - Add to `tests/property/` for property-based testing and fuzz testing
4. **Debug tests** - Add to `tests/debug/` only for temporary debugging (should be cleaned up regularly)

### Best Practices:
- **Always prefer unit tests in source files** for testing individual components
- **Use integration tests** when you need to test the public API or multiple components together
- **Unit tests have access to private functions**, integration tests only test public APIs
- **Follow existing naming conventions** and include descriptive test names