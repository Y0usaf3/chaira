# Versioned Type System Strategy for Scalable Data Storage

## Executive Summary

The current field/record type system lacks version tracking, which causes scaling issues for applications with evolving schemas. This document outlines a new strategy that introduces **Type Versioning**, **Schema Snapshots**, and **Gradual Migration** to enable safe, auditable schema evolution.

---

## Problems with Current Architecture

### 1. **No Version Tracking**
- `FieldConfig` has no version metadata
- When a field type changes, there's no history of what it was before
- Impossible to trace data lineage or understand why migration failed

### 2. **No Schema Snapshots**
- Records don't know which field schema version they were created with
- Data incompatibility issues are only discovered at read time
- No way to batch migrate data across versions

### 3. **Limited Migration Strategy**
- `MigrationStrategy` only has 4 states: Safe, Risky, Destructive, NoOp
- No tracking of which records have been migrated
- No rollback capability

### 4. **Type Enforcement is Scattered**
- Type validation lives in `cell.rs` with 700+ lines of ad-hoc code
- No central type registry or validation engine
- Hard to add new field types without modifying multiple files

### 5. **Record-to-Field Coupling is Loose**
- `Record` stores `cells: HashMap<String, CellValue>` with no field schema reference
- Can't validate cell contents without querying the field separately
- No enforcement of constraints at storage time

---

## Proposed Strategy: Versioned Type System (VTS)

### Phase 1: Type Versioning

#### 1.1 Add Version to FieldConfig

```rust
#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub id: Option<FieldId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub config: FieldConfig,
    pub config_version: u32,           // NEW: Track version of this config
    pub is_primary: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub name: String,
    pub order: u32,
    pub description: Option<String>,
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConfigVersion {
    pub field_id: FieldId,
    pub version: u32,
    pub config: FieldConfig,
    pub introduced_at: Datetime,
    pub deprecated_at: Option<Datetime>,
    pub migration_strategy: MigrationStrategy,
    pub previous_version: Option<u32>,
    pub change_description: String,
}
```

**Benefits:**
- Full audit trail of field configuration changes
- Can always reconstruct historical data state
- Enables rollback to previous versions

#### 1.2 Extend MigrationStrategy

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationStrategy {
    Safe,                      // No data loss, always compatible
    SafeWithDefault {          // Safe but needs default for missing values
        default_value: String,
    },
    Risky,                     // Possible data loss, requires review
    RiskyWithBackfill {        // Risky, but can apply transformation
        transformation: String, // e.g., "parse_as_int()", "uppercase()"
    },
    Destructive,               // Data will be lost/cleared
    DestructiveWithArchive,    // Archive old data before clearing
    NoOp,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MigrationState {
    Pending,                   // Not yet attempted
    InProgress {
        started_at: Datetime,
        progress: f32,         // 0.0 - 1.0
    },
    Completed {
        completed_at: Datetime,
        records_affected: u32,
        records_failed: u32,
    },
    Failed {
        error: String,
        failed_at: Datetime,
        rolled_back: bool,
    },
}
```

---

### Phase 2: Schema Snapshots

#### 2.1 Store Field Schema Reference in Records

```rust
#[derive(Debug, Clone, PartialEq, Eq, SurrealValue, serde::Serialize, serde::Deserialize)]
pub struct Record {
    pub id: Option<RecordId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub cells: HashMap<String, CellValue>,
    pub table: TableId,
    pub schema_version_id: SchemaVersionId,  // NEW: Which schema this was created with
    pub schema_snapshot: SchemaSnapshot,     // NEW: Immutable snapshot of fields
}

#[derive(Debug, Clone, PartialEq, Eq, SurrealValue, Serialize, Deserialize)]
pub struct SchemaSnapshot {
    pub version: u32,
    pub created_at: Datetime,
    pub fields: Vec<FieldSnapshot>,
    pub hash: String, // To detect if schema actually changed
}

#[derive(Debug, Clone, PartialEq, Eq, SurrealValue, Serialize, Deserialize)]
pub struct FieldSnapshot {
    pub field_id: FieldId,
    pub name: String,
    pub config_version: u32,
    pub config_hash: String,
}
```

**Benefits:**
- Records are self-describing with their schema
- Can detect schema drift (record has different schema than current field)
- Enables selective migration of records in specific schema versions

#### 2.2 Create Schema Version History Table

```
schema_versions table:
- id: SchemaVersionId
- table_id: TableId
- version: u32
- created_at: Datetime
- schema_snapshot: SchemaSnapshot
- change_summary: String
```

---

### Phase 3: Validation Engine

#### 3.1 Centralized Type Registry

```rust
pub struct TypeRegistry {
    validators: HashMap<String, Box<dyn TypeValidator>>,
    transformers: HashMap<String, Box<dyn TypeTransformer>>,
}

pub trait TypeValidator: Send + Sync {
    fn validate(&self, value: &Value, config: &FieldConfig) -> Result<(), ValidationError>;
    fn coerce(&self, value: &str, config: &FieldConfig) -> Result<Value, ValidationError>;
}

pub trait TypeTransformer: Send + Sync {
    fn transform(
        &self,
        value: &Value,
        from_config: &FieldConfig,
        to_config: &FieldConfig,
    ) -> Result<Value, TransformationError>;
}

pub enum ValidationError {
    TypeMismatch { expected: String, got: String },
    ConstraintViolation(String),
    ConversionFailed(String),
    InvalidFormat(String),
}
```

**Benefits:**
- Extensible: add new types without modifying core
- Single source of truth for validation
- Reusable transformers for migrations

#### 3.2 Constraint Enforcement

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FieldConstraints {
    pub is_primary: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub custom_validators: Vec<CustomValidator>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CustomValidator {
    MinLength(u32),
    MaxLength(u32),
    Pattern(String), // Regex
    MinValue(f64),
    MaxValue(f64),
    CustomFormula(String),
}

impl Record {
    pub fn validate_cell(
        &self,
        field_id: &FieldId,
        value: &CellValue,
        registry: &TypeRegistry,
    ) -> Result<(), ValidationError> {
        // Look up field in schema snapshot
        let field = self.schema_snapshot
            .fields
            .iter()
            .find(|f| f.field_id == field_id)
            .ok_or(ValidationError::FieldNotFound)?;
        
        // Validate against current config + constraints
        registry.validate(value, &field.config)?;
        Ok(())
    }
}
```

---

### Phase 4: Gradual Migration

#### 4.1 Migration Job Architecture

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationJob {
    pub id: MigrationJobId,
    pub field_id: FieldId,
    pub from_version: u32,
    pub to_version: u32,
    pub strategy: MigrationStrategy,
    pub state: MigrationState,
    pub batch_size: u32,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub estimated_records: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationBatch {
    pub job_id: MigrationJobId,
    pub batch_number: u32,
    pub record_ids: Vec<RecordId>,
    pub state: MigrationState,
    pub results: Vec<MigrationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationResult {
    pub record_id: RecordId,
    pub success: bool,
    pub old_value: Option<Value>,
    pub new_value: Option<Value>,
    pub error: Option<String>,
    pub transformed_at: Datetime,
}
```

#### 4.2 Migration Pipeline

```
1. Create MigrationJob
   ↓
2. Query records matching old schema version
   ↓
3. Batch into MigrationBatches (default 1000 records)
   ↓
4. For each batch:
   a. Transform values using TypeTransformer
   b. Validate against new FieldConfig
   c. Update Record.schema_version_id + schema_snapshot
   d. Record MigrationResult
   ↓
5. On success: Mark job as Completed
6. On failure: Rollback previous batch, mark job as Failed
```

**Benefits:**
- Process large migrations incrementally
- Avoid locking entire table
- Track what went wrong per record
- Can pause and resume jobs

#### 4.3 Safety Mechanisms

```rust
pub struct MigrationSafetyConfig {
    pub max_parallel_jobs: u32,
    pub batch_size: u32,
    pub rollback_on_first_failure: bool,
    pub create_backup_before_destructive: bool,
    pub notify_on_completion: bool,
}

impl MigrationJob {
    pub async fn execute(&self, config: MigrationSafetyConfig) -> Result<()> {
        // Check if destructive
        if matches!(self.strategy, MigrationStrategy::Destructive | MigrationStrategy::DestructiveWithArchive) {
            if config.create_backup_before_destructive {
                self.create_backup().await?;
            }
        }
        
        // Execute batches with error handling
        for batch in self.batches {
            match self.execute_batch(batch).await {
                Ok(_) => continue,
                Err(e) => {
                    if config.rollback_on_first_failure {
                        self.rollback_batch(batch).await?;
                        return Err(e);
                    }
                    // Otherwise record error and continue
                }
            }
        }
        Ok(())
    }
}
```

---

### Phase 5: Integration Points

#### 5.1 Field Creation - Add Initial Version

```rust
impl InsertField {
    pub fn to_field_with_version(self) -> Field {
        Field {
            id: None,
            created_at: None,
            updated_at: None,
            is_deleted: false,
            config: self.config,
            config_version: 1,        // Start at v1
            is_primary: self.is_primary,
            is_nullable: self.is_nullable,
            is_unique: self.is_unique,
            name: self.name,
            order: self.order,
            description: self.description,
        }
    }
}
```

#### 5.2 Field Update - Create Version History Entry

```rust
impl Field {
    pub async fn update(&mut self, patch: FieldPatch) -> Result<FieldConfigVersion> {
        let old_version = self.config_version;
        
        // Apply patch
        if let Some(new_config) = patch.config {
            let migration_strategy = self.config.get_migration_strategy(&new_config);
            
            // Create version history entry
            let version_entry = FieldConfigVersion {
                field_id: self.id.clone().unwrap(),
                version: old_version + 1,
                config: new_config.clone(),
                introduced_at: Datetime::now(),
                deprecated_at: None,
                migration_strategy,
                previous_version: Some(old_version),
                change_description: "User-initiated field config change".into(),
            };
            
            // Update field
            self.config = new_config;
            self.config_version = old_version + 1;
            self.updated_at = Some(Datetime::now());
            
            Ok(version_entry)
        } else {
            Err("No config change provided")
        }
    }
}
```

#### 5.3 Record Creation - Capture Schema Snapshot

```rust
impl InsertRecord {
    pub async fn to_record_with_snapshot(
        self,
        fields: Vec<Field>,
        schema_version: u32,
    ) -> Result<Record> {
        let field_snapshots = fields
            .iter()
            .map(|f| FieldSnapshot {
                field_id: f.id.clone().unwrap(),
                name: f.name.clone(),
                config_version: f.config_version,
                config_hash: serde_json::to_string(&f.config)
                    .and_then(|s| Ok(sha256(&s)))
                    .unwrap_or_default(),
            })
            .collect();
        
        let schema_snapshot = SchemaSnapshot {
            version: schema_version,
            created_at: Datetime::now(),
            fields: field_snapshots,
            hash: "calculated_hash".into(),
        };
        
        Record {
            id: None,
            created_at: None,
            updated_at: None,
            is_deleted: false,
            cells: self.cells,
            table: self.table,
            schema_version_id: schema_version.into(),
            schema_snapshot,
        }
    }
}
```

---

## Implementation Roadmap

### Week 1: Foundation
1. Add `FieldConfigVersion` table + schema
2. Add `config_version` to `Field`
3. Extend `MigrationStrategy` enum

### Week 2: Schema Snapshots
1. Add `schema_snapshot` to `Record`
2. Add `schema_version_id` tracking
3. Create `schema_versions` table

### Week 3: Validation Engine
1. Build `TypeRegistry` architecture
2. Implement `TypeValidator` trait
3. Implement `TypeTransformer` trait

### Week 4: Migration System
1. Create `MigrationJob` infrastructure
2. Build batch processing engine
3. Add rollback capability

### Week 5: Integration & Testing
1. Update all insert/update paths to use versioning
2. Write comprehensive migration tests
3. Add performance benchmarks

---

## Example: Safe Migration (Text → Number)

```
Before:
Field "age" is Text(SingleLine)
Records have cells["age"] = "25", "30", "not_a_number"

User wants to change to Number

1. System calculates migration strategy: Risky
   (because "not_a_number" will fail to parse)

2. System creates MigrationJob:
   - field_id: age
   - from_version: 1 → to_version: 2
   - strategy: RiskyWithBackfill { 
       transformation: "try_parse_int(value, default: 0)" 
     }

3. System scans records:
   - Record 1: "25" → 25 ✓
   - Record 2: "30" → 30 ✓
   - Record 3: "not_a_number" → 0 (applied default) ⚠️

4. System creates MigrationResult for each:
   - Logs which records used default
   - Allows manual review if needed

5. On approval:
   - Updates schema_version_id for each record
   - Updates schema_snapshot
   - Updates Field.config_version = 2
   - Creates FieldConfigVersion history entry

6. Rollback available if needed:
   - Restore all cells to previous Value type
   - Revert schema_snapshot to v1
```

---

## Example: Query-Time Version Detection

```rust
// When reading a record
impl Record {
    pub fn needs_migration(&self, current_field: &Field) -> bool {
        // Check if record's schema version is behind field's current version
        self.schema_snapshot.version < current_field.config_version
    }
    
    pub fn detect_schema_drift(&self) -> bool {
        // Check if field config hash matches snapshot
        self.schema_snapshot.fields
            .iter()
            .any(|snap| {
                // Query current field
                // Compare config_hash
                // If different, schema has drifted
                false // Simplified
            })
    }
}
```

---

## Benefits Summary

| Problem | Solution | Benefit |
|---------|----------|---------|
| No version tracking | `FieldConfigVersion` table | Full audit trail |
| Schema drift unknown | `schema_snapshot` in Record | Self-describing data |
| Scattered validation | `TypeRegistry` + traits | Extensible, centralized |
| No gradual migration | `MigrationJob` + batching | Safe, scalable evolution |
| No rollback | Migration history + state | Reversible changes |
| Constraint enforcement scattered | `FieldConstraints` struct | Centralized validation |

---

## Migration Path from Current System

### Phase 1: Backward Compatibility Layer
- Add optional `config_version` field (default = 1)
- Add optional `schema_snapshot` (auto-generate from current fields)
- Don't require immediate adoption

### Phase 2: Gradual Adoption
- New fields created with versioning
- Old fields lazily upgraded when updated
- Migration jobs optional

### Phase 3: Full Migration
- All records have schema snapshots
- All fields have version history
- Deprecate old validation paths
