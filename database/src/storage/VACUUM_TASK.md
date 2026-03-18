# Task: Implement Tombstone Vacuum

## Background

When documents are deleted, `PageLayout` marks their slot as a **tombstone** (`length == 0xFFFF`).
That space is never reclaimed — over time, deleted documents pile up as dead weight on pages.

The TODO in `storage_engine.rs:10` calls this out. Your job is to implement a `vacuum()` method
that compacts pages by removing tombstones and reclaiming free space.

---

## Files to Read First (in order)

1. `storage_engine.rs` — understand `insert`, `get`, `delete` flow
2. `page_layout.rs` — understand slot directory layout, tombstone marker (`0xFFFF`), `SlotEntry`
3. `buffer_pool.rs` — understand how to pin/unpin pages and mark them dirty
4. `page.rs` — understand the `Page` struct and `PAGE_SIZE`

---

## What to Implement

### Step 1 — Add `compact_page()` to `PageLayout`

In `page_layout.rs`, add a method:

```rust
pub fn compact_page(page: &mut Page) -> Result<(), DatabaseError>
```

It should:
- Read all non-tombstone slots
- Copy their document bytes contiguously starting from the data region start
- Rebuild the slot directory with updated offsets
- Update `free_space_offset` to reflect the new free space

**Hint:** The data region starts at `Self::get_header_size()`. Slots grow inward from the end of
the page. You need to pack live documents tightly from the start of the data region.

---

### Step 2 — Add `vacuum()` to `StorageEngine`

In `storage_engine.rs`, add a method:

```rust
pub fn vacuum(&mut self) -> Result<usize>
```

It should:
- Iterate over all page IDs (you can get total pages from `self.db_file`)
- For each page: pin it from the buffer pool, call `compact_page()`, mark it dirty, unpin it
- Return the number of pages compacted

---

### Step 3 — Write a Test

In `tests/integration/` (or a new `vacuum_test.rs`), write a test that:

1. Creates a storage engine with a temp file
2. Inserts ~10 documents
3. Deletes half of them
4. Records free space on a page (use `PageLayout::get_free_space()`)
5. Calls `vacuum()`
6. Asserts free space on that page increased

---

## Key Types and Methods to Know

| Location | Item | What it does |
|---|---|---|
| `page_layout.rs` | `TOMBSTONE_MARKER = 0xFFFF` | Marks a deleted slot |
| `page_layout.rs` | `SlotEntry::is_tombstone()` | Check if a slot is deleted |
| `page_layout.rs` | `get_free_space(page)` | Returns bytes available on a page |
| `page_layout.rs` | `read_slot_header(page)` | Returns `(slot_count, free_space_offset)` |
| `buffer_pool.rs` | `pin_page(page_id)` | Load a page into memory, lock it |
| `buffer_pool.rs` | `unpin_page(page_id, dirty)` | Release page, pass `true` if modified |
| `storage_engine.rs` | `delete_document(id)` | Sets tombstone — no compaction today |

---

## Definition of Done

- [ ] `compact_page()` implemented in `PageLayout`
- [ ] `vacuum()` implemented in `StorageEngine`, returns pages compacted
- [ ] At least one integration test passes that verifies free space increases after vacuum
- [ ] `cargo test` passes with no regressions
