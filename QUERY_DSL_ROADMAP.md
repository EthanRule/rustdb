# Elasticsearch Query DSL — Implementation Roadmap

## Overview

Elastic Query DSL is JSON-based. A query looks like:
```json
{
  "query": {
    "bool": {
      "must": [
        { "term": { "status": "active" } },
        { "range": { "age": { "gte": 18, "lte": 65 } } }
      ]
    }
  },
  "sort": [{ "age": "desc" }],
  "from": 0,
  "size": 10
}
```

Since your storage is ID-centric with no scan, every phase builds on the previous one. Don't skip ahead.

---

## Phase 1 — Storage Foundation
**Goal:** Make documents iterable. Everything else depends on this.

- Add `scan_all(&mut self) -> Result<Vec<(DocumentId, Document)>>` to `StorageEngine`
  - Iterate `page_id` from `0..page_count()`
  - For each page, read all non-tombstone slots and deserialize
- Add a `Collection` struct that wraps `StorageEngine` and owns a name + tracks its `DocumentId` list
  - `collections/mod.rs` — `Collection { name, engine, doc_ids: Vec<DocumentId> }`
  - `insert`, `scan`, `get_by_id` methods

**Test milestone:** Insert 5 docs, call `scan_all`, get all 5 back.

---

## Phase 2 — Query AST
**Goal:** Define Rust types that mirror the Elastic JSON structure using `serde`.

Create `src/query/mod.rs` with these types:

```
QueryRequest
  └── query: Option<Query>
  └── from: Option<usize>
  └── size: Option<usize>
  └── sort: Option<Vec<SortClause>>
  └── _source: Option<SourceFilter>

Query (enum)
  ├── Term { field, value }
  ├── Terms { field, values: Vec<Value> }
  ├── Range { field, gte, gt, lte, lt }
  ├── Match { field, query: String }
  ├── MatchAll
  ├── Exists { field }
  └── Bool { must, should, must_not, filter }
```

Use `serde_json` to deserialize the incoming JSON directly into these types. No hand-written parser needed — Elastic DSL is already JSON.

**Test milestone:** Deserialize a `bool` query JSON string into your AST without errors.

---

## Phase 3 — Predicate Evaluator
**Goal:** Given a `Query` and a `Document`, return `true`/`false`.

Create `src/query/evaluator.rs`:

- `fn matches(query: &Query, doc: &Document) -> bool`
- Handle each variant:
  - `Term` → `doc.get_path(field) == Some(value)`
  - `Range` → compare with `gte`/`lte`/`gt`/`lt` bounds using your `Value` ordering
  - `Match` → case-insensitive string contains (full-text comes later)
  - `MatchAll` → always `true`
  - `Exists` → `doc.get_path(field).is_some()`
  - `Bool` → `must` all pass AND `must_not` none pass AND (`should` is empty OR at least one passes)
  - `Terms` → field value is in the list

Your `Value` enum already has type-safe comparisons — lean on those.

**Test milestone:** Write unit tests for each query type against hand-crafted documents.

---

## Phase 4 — Query Executor
**Goal:** Run a full `QueryRequest` against a collection and return results.

Create `src/query/executor.rs`:

```
fn execute(request: &QueryRequest, collection: &mut Collection) -> QueryResult
```

Steps inside `execute`:
1. `scan_all()` to get every document
2. Filter with `evaluator::matches()`
3. Apply `sort` — implement `Value` ordering for `asc`/`desc`
4. Apply `from`/`size` pagination (slice the vec)
5. Apply `_source` field inclusion/exclusion
6. Return `QueryResult { hits: Vec<Document>, total: usize }`

**Test milestone:** Execute a `bool` query with range + term, get correct paginated results.

---

## Phase 5 — Full-Text Match
**Goal:** Make `match` queries actually useful.

Upgrade the `Match` evaluator:
- Tokenize both the query string and the field value (split on whitespace/punctuation)
- Score by token overlap — docs where more tokens match rank higher
- Add `_score` field to results
- Sort by `_score` when no explicit sort is given

Optional but valuable: add `match_phrase` (tokens must appear in order) and `multi_match` (match across multiple fields).

**Test milestone:** Insert docs with varying descriptions, `match` query returns them ranked by relevance.

---

## Phase 6 — Aggregations
**Goal:** Support the `aggs` block for analytics queries.

Add `aggs: Option<HashMap<String, Aggregation>>` to `QueryRequest`.

```
Aggregation (enum)
  ├── ValueCount { field }          → count non-null values
  ├── Avg / Sum / Min / Max { field }
  ├── Terms { field, size }         → bucket by unique values
  └── Range { field, ranges: Vec<{ from, to }> }
```

Create `src/query/aggregator.rs`:
- Run aggregations on the **filtered** doc set (after query, before pagination)
- Return `AggResult { buckets/value }` per named aggregation

**Test milestone:** `terms` agg on a `status` field returns correct per-value counts.

---

## Phase 7 — Indexing (Performance)
**Goal:** Stop doing full scans for common query patterns.

This is an optimization phase — your queries should already be correct, just slow on large datasets.

- Add `src/storage/index.rs` — a `BTreeMap<Value, Vec<DocumentId>>` per indexed field
- Persist indexes to their own page type (your `PageType::Index` already exists in the type system)
- Query executor checks: if a `term` or `range` query targets an indexed field, use the index instead of scanning
- Invalidate index entries on `update`/`delete`

**Test milestone:** Index on `age`, range query uses index, verify same results as scan but faster (benchmark it).

---

## Phase 8 — GUI/CLI Integration
**Goal:** Make queries usable from your existing UI and a REPL.

- Add a **Query tab** to the egui GUI: JSON text input → run → display hits + aggregations
- Add a **CLI binary** (`cargo run --bin db_query`) that reads query JSON from stdin or a file
- Display `_score`, `total`, and per-field aggregation results

---

## Suggested File Layout

```
src/
  query/
    mod.rs          ← QueryRequest, Query, SortClause AST types
    evaluator.rs    ← matches(query, doc) -> bool
    executor.rs     ← execute(request, collection) -> QueryResult
    aggregator.rs   ← run_aggs(aggs, docs) -> AggResults
  collections/
    mod.rs          ← Collection struct wrapping StorageEngine
  storage/
    index.rs        ← Phase 7 only
```

---

## Recommended Order

| Phase | Unlock |
|---|---|
| 1 — Scan + Collections | Unblocks everything |
| 2 — Query AST | Unblocks 3 & 4 |
| 3 — Evaluator | Unblocks 4 |
| 4 — Executor | First working queries end-to-end |
| 5 — Full-text | Relevance scoring |
| 6 — Aggregations | Analytics |
| 7 — Indexing | Performance at scale |
| 8 — UI/CLI | Usability |

Start with Phase 1 — nothing else is possible without a scan.
