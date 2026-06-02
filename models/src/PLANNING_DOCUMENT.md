# Type System Redesign - Planning Document

## High-Level Vision

We need to:
1. Store cells as a **vector of flexible objects** (not HashMap)
2. Add **verification at create/update time**
3. Optimize **serialization size**
4. Track **schema versions**
5. Enable **gradual migrations**

---

## Current Problem

```
Record {
  cells: HashMap<String, CellValue> {
    "field_1": CellValue {
      id: CellId,
      created_at: Datetime,
      updated_at: Datetime,
      value: SingleLine("hello")  ← Buried under metadata
    }
  }
}
```

**Issues:**
- HashMap requires string keys (slower lookups)
- Metadata stored per cell (bloated)
- No schema validation before storage
- Hard to track which version each cell was created with

---

## Proposed New Structure

### Record Storage Layer

```
Record {
  id: RecordId,
  created_at: Datetime,
  updated_at: Datetime,
  table_id: TableId,
  schema_version: u32,
  schema_snapshot: SchemaSnapshot,
  
  // NEW: Vector-based storage
  cells: Vec<CellEntry>,
}

CellEntry {
  field_id: FieldId,    // Which field this is
  value: Value,         // The actual data
  // Metadata moved out ↓
}

// Metadata stored separately at record level
CellMetadataIndex {
  entries: Vec<CellMetadata>  // Indexed by position
}

CellMetadata {
  cell_id: CellId,
  created_at: Datetime,
  updated_at: Datetime,
}
```

**Why Vector?**
- Maintains order consistent with schema
- Faster iteration for serialization
- Can use fixed indices instead of string keys
- Cache-friendly (linear memory layout)

---

## SurrealDB Schema Design

### Current (Problems)

```surql
DEFINE TABLE record TYPE OBJECT;
DEFINE FIELD cells ON TABLE record TYPE object;
  // cells is just... an object
  // No structure validation
  // SurrealDB doesn't know shape
```

### New (Proposed)

```surql
-- Define the record table
DEFINE TABLE record TYPE OBJECT FLEXIBLE;

-- NEW: Define cells as array of flexible objects
-- This enforces that cells is always an array
DEFINE FIELD cells ON TABLE record 
  TYPE array 
  ASSERTIONS [
    type::is_array($value),
    array::all($value, |$cell| type::is_object($cell) AND $cell CONTAINS "field_id" AND $cell CONTAINS "value")
  ]
  VALUE [];

-- Each cell object must have structure
DEFINE FIELD cells[*].field_id ON TABLE record 
  TYPE string 
  ASSERT type::is_string($value);

DEFINE FIELD cells[*].value ON TABLE record 
  TYPE any  -- Flexible because we support all Value types
  ASSERT true;  -- Client-side validation happens first

-- Separate metadata index
DEFINE FIELD cell_metadata ON TABLE record 
  TYPE array 
  VALUE [];

DEFINE FIELD cell_metadata[*].cell_id ON TABLE record 
  TYPE string;

DEFINE FIELD cell_metadata[*].created_at ON TABLE record 
  TYPE datetime;

DEFINE FIELD cell_metadata[*].updated_at ON TABLE record 
  TYPE datetime;

-- Schema tracking
DEFINE FIELD schema_version ON TABLE record TYPE number;
DEFINE FIELD schema_snapshot ON TABLE record TYPE object;
```

---

## Data Flow Diagrams

### Create Record Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. Client sends InsertRecord                             │
│    {table_id, cells: [(field_1, value_1), ...]}         │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 2. Server: Fetch current Field schema                    │
│    - Get all fields for this table                       │
│    - Get latest schema_version                          │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 3. Verification Layer (FAIL FAST)                        │
│    for each (field_id, value) in cells:                 │
│      - Find field by id                                 │
│      - value.verify(&field.config)?  ← CRITICAL        │
│      - size_constraint.validate()?                      │
│      - constraints.validate()?                          │
│    if any fail: return Error immediately ✗              │
└──────────────────────┬──────────────────────────────────┘
                       │ (All verified ✓)
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 4. Build Schema Snapshot                                 │
│    SchemaSnapshot {                                      │
│      version: 1,                                         │
│      created_at: now(),                                  │
│      fields: [FieldSnapshot {...}, ...]                 │
│      hash: sha256(schema)                               │
│    }                                                     │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 5. Build Record                                          │
│    Record {                                              │
│      id: generated,                                      │
│      created_at: now(),                                  │
│      cells: [                                            │
│        CellEntry { field_id: "1", value: "hello" },    │
│        CellEntry { field_id: "2", value: 42 },         │
│      ],                                                  │
│      cell_metadata: [                                    │
│        {cell_id: "cell|1", created_at, updated_at},    │
│        {cell_id: "cell|2", created_at, updated_at},    │
│      ],                                                  │
│      schema_version: 1,                                  │
│      schema_snapshot: {...}                             │
│    }                                                     │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 6. Serialize to JSON (Compact)                           │
│    {                                                     │
│      "id": "record|abc",                               │
│      "cells": [                                          │
│        {"field_id": "f1", "value": "hello"},           │
│        {"field_id": "f2", "value": 42}                 │
│      ],                                                  │
│      "cell_metadata": [...]                            │
│    }                                                     │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 7. Store in SurrealDB                                    │
│    INSERT INTO record {                                  │
│      id, created_at, cells, cell_metadata, ...         │
│    };                                                    │
└─────────────────────────────────────────────────────────┘
```

### Update Record Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. Client sends RecordPatch                              │
│    {record_id, changed_cells: [(field_id, new_value)]}  │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 2. Fetch existing Record from DB                         │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 3. Verification Layer (FAIL FAST)                        │
│    for each (field_id, new_value) in changed_cells:    │
│      - value.verify(&field.config)?                     │
│      - size_constraint.validate()?                      │
│    if any fail: return Error immediately ✗              │
└──────────────────────┬──────────────────────────────────┘
                       │ (All verified ✓)
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 4. Update cells vector                                   │
│    for each (field_id, new_value):                      │
│      find_by_field_id(cells, field_id).value = new_val │
│      update cell_metadata[idx].updated_at = now()       │
└──────────────────────┬──────────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────────┐
│ 5. Store in DB                                           │
│    UPDATE record SET                                     │
│      cells = [...],                                      │
│      cell_metadata = [...],                              │
│      updated_at = now()                                  │
│    WHERE id = record_id;                                │
└─────────────────────────────────────────────────────────┘
```

---

## Serialization Comparison

### Before (HashMap-based, bloated)

```json
{
  "id": "record|xyz123",
  "created_at": "2024-06-02T10:00:00Z",
  "updated_at": "2024-06-02T10:00:00Z",
  "cells": {
    "field_1_name": {
      "id": "cell|abc1",
      "created_at": "2024-06-02T10:00:00Z",
      "updated_at": "2024-06-02T10:00:00Z",
      "value": "John"
    },
    "field_2_name": {
      "id": "cell|abc2",
      "created_at": "2024-06-02T10:00:00Z",
      "updated_at": "2024-06-02T10:00:00Z",
      "value": 25
    },
    "field_3_name": {
      "id": "cell|abc3",
      "created_at": "2024-06-02T10:00:00Z",
      "updated_at": "2024-06-02T10:00:00Z",
      "value": "john@example.com"
    }
  }
}
```

**Size: ~850 bytes** (for 3 simple values!)

### After (Vector-based, optimized)

```json
{
  "id": "record|xyz123",
  "created_at": "2024-06-02T10:00:00Z",
  "updated_at": "2024-06-02T10:00:00Z",
  "cells": [
    {"field_id": "f1", "value": "John"},
    {"field_id": "f2", "value": 25},
    {"field_id": "f3", "value": "john@example.com"}
  ],
  "cell_metadata": [
    {"cell_id": "cell|abc1", "created_at": "2024-06-02T10:00:00Z", "updated_at": "2024-06-02T10:00:00Z"},
    {"cell_id": "cell|abc2", "created_at": "2024-06-02T10:00:00Z", "updated_at": "2024-06-02T10:00:00Z"},
    {"cell_id": "cell|abc3", "created_at": "2024-06-02T10:00:00Z", "updated_at": "2024-06-02T10:00:00Z"}
  ]
}
```

**Size: ~450 bytes** (47% reduction!)

---

## Rust Struct Evolution

### Phase 1: Current (Baseline)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: Option<RecordId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub cells: HashMap<String, CellValue>,  // ← OLD
    pub table: TableId,
}
```

### Phase 2: Transition (Both supported)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: Option<RecordId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cells_legacy: Option<HashMap<String, CellValue>>,  // ← Keep for migration
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cells: Option<Vec<CellEntry>>,  // ← NEW
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cell_metadata: Option<CellMetadataIndex>,  // ← NEW
    
    pub table: TableId,
    pub schema_version_id: u32,      // ← NEW (from TYPE_SYSTEM_STRATEGY.md)
    pub schema_snapshot: SchemaSnapshot,  // ← NEW
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellEntry {
    pub field_id: FieldId,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadataIndex {
    pub entries: Vec<CellMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellMetadata {
    pub cell_id: CellId,
    pub created_at: Datetime,
    pub updated_at: Datetime,
}
```

### Phase 3: Final (New only)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: Option<RecordId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub cells: Vec<CellEntry>,           // ← Only new format
    pub cell_metadata: CellMetadataIndex,
    pub table: TableId,
    pub schema_version_id: u32,
    pub schema_snapshot: SchemaSnapshot,
}
```

---

## Verification Logic Tree

```
Record::create(insert, fields)
├─ insert.verify(fields)
│  ├─ for each (field_id, cell_value) in insert.cells:
│  │  ├─ Find field in fields by id
│  │  │  └─ if not found → FieldNotFound error ✗
│  │  │
│  │  ├─ cell_value.value.verify(&field.config)
│  │  │  ├─ Type check
│  │  │  │  └─ match value enum to config enum ✗ if mismatch
│  │  │  │
│  │  │  ├─ Size check
│  │  │  │  ├─ for SingleLine: len <= max_length ✗
│  │  │  │  ├─ for LongText: size_mb <= max_mb ✗
│  │  │  │  └─ for Attachment: total_size <= max_mb ✗
│  │  │  │
│  │  │  ├─ Constraint check
│  │  │  │  ├─ if field.is_nullable == false && value is None ✗
│  │  │  │  ├─ if field.is_unique && value exists elsewhere ✗
│  │  │  │  └─ if field.is_primary && value is None ✗
│  │  │  │
│  │  │  └─ Format check (type-specific)
│  │  │     ├─ Email: valid email format ✗
│  │  │     ├─ URL: valid URL format ✗
│  │  │     └─ Phone: valid phone number ✗
│  │  │
│  │  └─ all pass → ✓
│  │
│  └─ if any cell fails → Return error immediately ✗
│
├─ build_schema_snapshot(fields) → SchemaSnapshot
│
├─ create Record struct
│  ├─ cells = Vec<CellEntry>
│  ├─ cell_metadata = CellMetadataIndex
│  └─ schema_snapshot = built above
│
└─ store in database ✓
```

---

## Size Constraints Registry

```
Field Type           │ Default Limit    │ Rationale
─────────────────────┼──────────────────┼─────────────────────
SingleLine           │ 500 bytes        │ Quick field values
LongText             │ 1 MB             │ Rich content
Email                │ 255 bytes        │ Email standard
URL                  │ 2 KB             │ URLs can be long
Phone                │ 20 bytes         │ E164 format
Number               │ N/A (8 bytes)    │ Fixed size
Decimal              │ N/A (8 bytes)    │ Fixed size
Rating               │ N/A (1 byte)     │ 0-10
Percent              │ N/A (4 bytes)    │ Fixed size
Currency             │ N/A (8 bytes)    │ Fixed size
Date                 │ N/A (16 bytes)   │ Fixed size
Attachment           │ 100 MB           │ File storage
JSON                 │ 5 MB             │ Complex data
Formula              │ N/A              │ Computed
AutoNumber           │ N/A              │ Generated
```

---

## Tasks Breakdown

### Week 1: Foundation
- [ ] Define new Record/CellEntry/CellMetadata structs (Phase 2)
- [ ] Update SurrealDB schema definition
- [ ] Implement TypeVerifier trait
- [ ] Add verification logic to Value::verify()

### Week 2: Record Operations
- [ ] Implement Record::create() with verification
- [ ] Implement Record::update() with verification
- [ ] Add backward compatibility layer (handle both old/new format)
- [ ] Write tests for verification

### Week 3: Optimization
- [ ] Remove CellValue per-cell metadata (move to CellMetadataIndex)
- [ ] Update serialization logic
- [ ] Measure size reduction
- [ ] Add serialization benchmarks

### Week 4: Schema Integration
- [ ] Integrate SchemaSnapshot from TYPE_SYSTEM_STRATEGY.md
- [ ] Add schema version tracking to Record
- [ ] Update migration logic
- [ ] End-to-end testing

### Week 5: Migration & Cleanup
- [ ] Remove legacy HashMap cells format
- [ ] Final benchmarks (performance, size)
- [ ] Documentation
- [ ] User-facing API updates

---

## API Examples (After Implementation)

### Creating a Record

```rust
// User code
let insert = InsertRecord::new(table_id, vec![
    (field_id_1, Value::SingleLine(...)),
    (field_id_2, Value::Number(...)),
]);

// Verification + creation
let record = Record::create(insert, &fields).await?;

// Internally:
// 1. Verifies each value against field config
// 2. Creates schema snapshot
// 3. Stores as Vec<CellEntry> (compact)
```

### Reading a Record

```rust
// User gets record from DB
let record = db.get_record(record_id).await?;

// Access cells by field_id
for cell_entry in &record.cells {
    let metadata = record.cell_metadata.get(cell_entry.field_id)?;
    println!("Field {}: {:?} (created: {})", 
        cell_entry.field_id, 
        cell_entry.value,
        metadata.created_at
    );
}
```

### Updating a Record

```rust
let patch = RecordPatch::new(vec![
    (field_id_1, Value::SingleLine(...)),
]);

// Verification + update
record.update(patch, &fields).await?;

// Internally:
// 1. Verifies new values
// 2. Updates Vec entries
// 3. Updates metadata timestamps
```

---

## Questions to Answer

1. **Should field_id be a String or number?**
   - String: Human readable, flexible
   - Number: Smaller storage, faster lookup
   - Recommendation: Use field position as index, store mapping once

2. **Where to store field_id → position mapping?**
   - In Record: Duplicated, but fast lookups
   - In SchemaSnapshot: Single source of truth
   - In Field struct: Schema level
   - Recommendation: SchemaSnapshot, use it for validation

3. **Should cell_metadata be optional?**
   - Some cells might not need tracking
   - Could reduce size further
   - Recommendation: Always include (consistency)

4. **How to handle nullable cells?**
   - Empty Vec element? 
   - Null in JSON?
   - Not include the entry?
   - Recommendation: Use Value::Null enum variant

5. **Backward compatibility window?**
   - How long to support old HashMap format?
   - Phase 2 transition period?
   - Recommendation: 2-3 major versions (3 months)

---

## Visual: Before vs After

```
BEFORE:
┌─────────────────────────────────────────┐
│ Record                                   │
├─────────────────────────────────────────┤
│ cells: HashMap {                         │
│   "field_name_1": {                      │ ← String key (slow)
│     id, created_at, updated_at,          │ ← Repeated metadata
│     value: "hello"                       │
│   },                                     │
│   "field_name_2": {                      │ ← String key (slow)
│     id, created_at, updated_at,          │ ← Repeated metadata
│     value: 42                            │
│   }                                      │
│ }                                        │
│                                          │
│ Storage: ~850 bytes                      │
│ Lookup: HashMap lookup (O(n))            │
└─────────────────────────────────────────┘

AFTER:
┌──────────────────────────────────────┐
│ Record                                │
├──────────────────────────────────────┤
│ cells: Vec [                          │
│   {field_id: "f1", value: "hello"},  │ ← Indexed, compact
│   {field_id: "f2", value: 42}        │ ← No metadata clutter
│ ]                                    │
│                                      │
│ cell_metadata: Vec [                 │
│   {cell_id, created_at, updated_at}, │ ← Metadata once
│   {cell_id, created_at, updated_at}  │
│ ]                                    │
│                                      │
│ Storage: ~450 bytes                  │
│ Lookup: Vec index (O(1))             │
└──────────────────────────────────────┘
```

---

## Next Steps

1. ✅ Read this document thoroughly (get familiar with the vision)
2. ⬜ Sketch on paper/whiteboard (physical planning!)
3. ⬜ Create Rust structs for Phase 2
4. ⬜ Update SurrealDB schema
5. ⬜ Implement TypeVerifier trait
6. ⬜ Add verification to Record::create/update
