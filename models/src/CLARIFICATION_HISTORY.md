# Clarification: Record History vs Schema History

## Short Answer

**NO** - We are NOT storing previous cell values by default. We only track:
- When the cell was last modified (`updated_at` in metadata)
- The current value

However, we CAN add audit logging if you want full history.

---

## What We're Currently Tracking

### ✅ We Track These:

```rust
CellMetadata {
    cell_id: CellId,           // Unique ID for this cell
    created_at: Datetime,      // When this cell was first created
    updated_at: Datetime,      // When it was LAST modified ← This tells us "something changed"
}
```

So we know:
- Cell was created on June 1st
- Cell was last modified on June 2nd
- But we DON'T know what the old value was

### ❌ We DON'T Track By Default:

```rust
// We DON'T store this:
pub struct CellHistory {
    cell_id: CellId,
    version: u32,
    old_value: Value,  // ← What it was before
    new_value: Value,  // ← What it is now
    changed_by: UserId,
    changed_at: Datetime,
    change_reason: Option<String>,
}
```

---

## Example: What Happens When You Edit

### Current State (Before Edit)

```
Record with id="rec|123"
├─ cells[0] = {field_id: "name", value: "John"}
│  └─ metadata: {created_at: June 1, updated_at: June 1}
│
└─ cells[1] = {field_id: "age", value: 25}
   └─ metadata: {created_at: June 1, updated_at: June 1}
```

### You Edit: Change name to "Jane"

```rust
let patch = RecordPatch::new(vec![
    ("name".to_string(), Value::SingleLine("Jane")),
]);

record.update(patch, &fields).await?;
```

### After Edit

```
Record with id="rec|123"
├─ cells[0] = {field_id: "name", value: "Jane"}  ← Changed!
│  └─ metadata: {created_at: June 1, updated_at: June 2 10:00:00}  ← Updated timestamp
│
└─ cells[1] = {field_id: "age", value: 25}
   └─ metadata: {created_at: June 1, updated_at: June 1}
```

**What we know:**
- ✓ Name was changed on June 2
- ✗ We DON'T know it was "John" before

---

## If You WANT Cell History

You need to explicitly create an audit table:

```rust
// NEW: Optional history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellValueHistory {
    pub id: CellHistoryId,
    pub cell_id: CellId,
    pub record_id: RecordId,
    pub field_id: FieldId,
    pub old_value: Option<Value>,
    pub new_value: Value,
    pub changed_at: Datetime,
    pub changed_by: Option<UserId>,  // Who made the change
    pub change_reason: Option<String>,  // Why
}

// When updating a cell:
impl Record {
    pub async fn update(
        &mut self, 
        patch: RecordPatch, 
        fields: &[Field],
        audit_enabled: bool,  // NEW: Optional
    ) -> Result<()> {
        // Verify
        patch.verify(fields).await?;
        
        // If audit is enabled, record old values
        if audit_enabled {
            for (field_id, new_value) in &patch.changed_cells {
                // Find old value
                if let Some(cell) = self.cells.iter().find(|c| c.field_id == field_id) {
                    let history = CellValueHistory {
                        id: CellHistoryId::new(),
                        cell_id: cell_metadata.get(field_id)?.cell_id.clone(),
                        record_id: self.id.clone().unwrap(),
                        field_id: field_id.clone(),
                        old_value: Some(cell.value.clone()),  // ← Save old value
                        new_value: new_value.clone(),
                        changed_at: Datetime::now(),
                        changed_by: None, // Get from context
                        change_reason: None,
                    };
                    
                    db.insert_cell_history(&history).await?;
                }
            }
        }
        
        // Apply changes
        self.cells.iter_mut()
            .filter_map(|c| {
                patch.changed_cells.iter()
                    .find(|(fid, _)| fid == &c.field_id)
                    .map(|(_, val)| (c, val))
            })
            .for_each(|(cell, val)| cell.value = val.clone());
        
        Ok(())
    }
}

// Usage
record.update(patch, &fields, audit_enabled: true).await?;
// Now old values are saved!
```

---

## What IS Preserved (Schema Version-wise)

The confusing part: We DO preserve **field schema information** at record creation time:

```rust
// When record is created, we KEEP:
pub struct SchemaSnapshot {
    pub version: u32,
    pub created_at: Datetime,
    pub fields: Vec<FieldSnapshot>,
    pub hash: String,
}

pub struct FieldSnapshot {
    pub field_id: FieldId,
    pub name: String,
    pub config_version: u32,        // ← What version field was at
    pub config_hash: String,        // ← What the config looked like
}
```

So if field "age" was:
- June 1: Number type (version 1)
- June 2: User changes to Text type (version 2)

The record created on June 1 will still have in its snapshot:
```
{field_id: "age", config_version: 1, config_hash: hash_of_number}
```

This allows us to know: "This record's 'age' field was Number at creation time"

But it does NOT preserve the actual cell values over time.

---

## Decision: What Do You Want?

### Option A: Lightweight (Current Proposal)
- ✓ Small storage
- ✓ Fast updates
- ✗ No value history
- ✗ Can't see old values

```
Space: 50 MB for 1M records
```

### Option B: With Audit Trail
- ✓ Full value history
- ✓ Can see all changes
- ✓ Can see who changed what
- ✗ 3-5x larger storage
- ✗ Slower updates (write to history table)

```
Space: 200-300 MB for 1M records (3-5x more)
```

### Option C: Hybrid (Recommended for scaling)
- ✓ Value history for first N days
- ✓ Then archive/delete old history
- ✓ Schema history kept forever
- ✓ Reasonable storage

```
Space: 100 MB for 1M records (2x base)
Retention: Keep 30 days of history
```

---

## Real World Comparison

### Example 1: Slack Message Edit

```
Original: "Let's have a meeting"
Edit 1:   "Let's have a meeting tomorrow"
Edit 2:   "Let's have a meeting tomorrow at 2pm"

Slack stores: All versions + who edited + when
Cost: Higher storage, but gives full transparency
```

### Example 2: Spreadsheet Cell Edit

```
Original: "100"
Edit 1:   "150"
Edit 2:   "125"

Google Sheets stores: All versions + timestamps
Cost: Significant storage, but part of value prop
```

### Example 3: Database Record Update (Traditional)

```
Original: age = 25
Edit 1:   age = 26

Most databases store: Just current value
Cost: Minimal storage
You need: Separate audit table if you want history
```

---

## Implementation Roadmap

### Phase 1: NO History (Current Plan)
```rust
Record {
    cells: Vec<CellEntry>,
    cell_metadata: CellMetadataIndex,  // Only created_at, updated_at
}

// Storage: Minimal
// You can see: "Last modified on June 2"
// You cannot see: "What was it before"
```

### Phase 2: Optional Audit (If needed)
```rust
// NEW table: cell_value_history
pub struct CellValueHistory {
    old_value: Value,
    new_value: Value,
    changed_at: Datetime,
}

// Usage: record.update(patch, fields, enable_audit: true)
// Storage: +3-5x for audit trail
```

### Phase 3: Smart Retention (Scaling)
```rust
// Keep history for 30 days, then archive
pub async fn cleanup_old_history() {
    let cutoff = Datetime::now() - Duration::days(30);
    db.query(
        "DELETE FROM cell_value_history WHERE changed_at < $cutoff",
        vec![("$cutoff", cutoff)]
    ).await?;
}
```

---

## Summary Table

| Feature | Option A | Option B | Option C |
|---------|----------|----------|----------|
| **Storage** | 1x | 5x | 2x |
| **Speed** | Fast | Slow | Fast |
| **See old values?** | ✗ | ✓ | ✓ (30 days) |
| **See who changed?** | ✗ | ✓ | ✓ (30 days) |
| **Best for** | Internal use | Compliance | Balanced |

---

## My Recommendation

**Start with Option A** (no history), because:
1. Simpler implementation
2. Faster performance
3. Less storage
4. Can add history later if needed

If users ask for change history later, add Option C (hybrid with retention).

---

## Code Example: How to Know If Value Changed

Even WITHOUT storing history, you can still know THAT it changed:

```rust
impl Record {
    pub fn when_was_field_last_modified(&self, field_id: &FieldId) -> Option<Datetime> {
        self.cell_metadata
            .get_by_field_id(field_id)
            .map(|meta| meta.updated_at)
    }
    
    pub fn has_field_been_modified(&self, field_id: &FieldId) -> bool {
        self.cell_metadata
            .get_by_field_id(field_id)
            .map(|meta| meta.updated_at > meta.created_at)
            .unwrap_or(false)
    }
}

// Usage
if record.has_field_been_modified(&field_id) {
    println!("Name field was modified");
    println!("Last modified: {}", record.when_was_field_last_modified(&field_id).unwrap());
} else {
    println!("Name field has never been changed since creation");
}

// Output:
// Name field was modified
// Last modified: 2024-06-02 10:00:00
```

---

## If You Want Full Audit Trail Right Away

Here's the complete implementation:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellValueHistory {
    pub id: CellHistoryId,
    pub cell_id: CellId,
    pub record_id: RecordId,
    pub field_id: FieldId,
    pub old_value: Value,
    pub new_value: Value,
    pub changed_at: Datetime,
    pub changed_by: Option<UserId>,
    pub change_reason: Option<String>,
}

impl Record {
    pub async fn update_with_audit(
        &mut self,
        patch: RecordPatch,
        fields: &[Field],
        user_id: Option<UserId>,
        reason: Option<String>,
    ) -> Result<()> {
        // 1. Verify
        patch.verify(fields).await?;
        
        // 2. Create history entries for all changes
        for (field_id, new_value) in &patch.changed_cells {
            if let Some(cell) = self.cells.iter().find(|c| c.field_id == field_id) {
                let history = CellValueHistory {
                    id: CellHistoryId::new(),
                    cell_id: self.cell_metadata.get_by_field_id(field_id)?.cell_id.clone(),
                    record_id: self.id.clone().unwrap(),
                    field_id: field_id.clone(),
                    old_value: cell.value.clone(),
                    new_value: new_value.clone(),
                    changed_at: Datetime::now(),
                    changed_by: user_id,
                    change_reason: reason.clone(),
                };
                
                db.insert(&history).await?;
            }
        }
        
        // 3. Apply changes
        for (field_id, new_value) in patch.changed_cells {
            if let Some(cell) = self.cells.iter_mut().find(|c| c.field_id == field_id) {
                cell.value = new_value;
            }
        }
        
        // 4. Update timestamp
        self.updated_at = Some(Datetime::now());
        
        Ok(())
    }
}

// Query history
pub async fn get_cell_history(
    record_id: &RecordId,
    field_id: &FieldId,
) -> Result<Vec<CellValueHistory>> {
    db.query(
        "SELECT * FROM cell_value_history 
         WHERE record_id = $record_id 
         AND field_id = $field_id 
         ORDER BY changed_at DESC",
        vec![
            ("$record_id", record_id.to_string()),
            ("$field_id", field_id.to_string()),
        ]
    ).await
}

// Usage
record.update_with_audit(
    patch,
    &fields,
    Some(user_id),
    Some("Admin corrected typo".to_string()),
).await?;

// Later, see history
let history = get_cell_history(&record_id, &field_id).await?;
for entry in history {
    println!(
        "{}: {} → {} (by {:?})",
        entry.changed_at,
        entry.old_value,
        entry.new_value,
        entry.changed_by
    );
}
```

---

## Key Distinction

```
SCHEMA VERSIONING                    CELL VALUE HISTORY
═══════════════════════════════════════════════════════════
"What type was field?"               "What was the cell value?"

Preserved automatically:              Optional to preserve:
✓ Field config version                ? Old cell values
✓ Field config hash                   ? Who changed it
✓ When field was created              ? When it changed
                                      ? Why it changed

We ALWAYS know this:                 We DON'T know (unless auditing):
✓ Field was Text on June 1            ✗ Value was "John" on June 1
✓ Field became Number on June 2       ✗ Changed to "Jane" on June 2
✗ But old cell values are lost        ✗ By whom and when
```

---

**Bottom line:** Choose what makes sense for your app:
- **Internal tool?** → No history (lighter, faster)
- **User-facing app?** → Add history (transparency, trust)
- **Compliance required?** → Full audit trail (legal protection)
