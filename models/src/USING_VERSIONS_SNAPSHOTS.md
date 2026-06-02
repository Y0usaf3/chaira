# Using Versions and Snapshots - Practical Guide

## Overview

You have three key pieces of information:
1. **Current Field Config** - What the field type is RIGHT NOW
2. **Record's Schema Snapshot** - What the field type was when this record was CREATED
3. **Field Config Version** - Which version the current field is at
4. **Migration Job** - Tracking the transition between versions

This guide shows how to use them together.

---

## Problem Scenarios

### Scenario 1: Reading a Record (Simple Case)

**Situation:** You fetch a record and want to display it

```
Field "age" current state: Number(Integer)
Record created: 2 months ago
Record's schema_snapshot says: Text(SingleLine)
```

**What does this mean?**
- The field TYPE CHANGED since record creation
- Record data is still stored as Text
- Need to either: display as-is OR migrate on-the-fly

#### Option A: Display As-Is (Read-Only)

```rust
impl Record {
    pub fn get_cell(&self, field_id: &FieldId) -> Result<DisplayValue> {
        // Find cell in this record
        let cell_entry = self.cells
            .iter()
            .find(|c| c.field_id == field_id)
            .ok_or(RecordError::CellNotFound)?;
        
        // Get metadata
        let metadata = self.cell_metadata
            .get_by_field_id(field_id)?;
        
        // Find the field snapshot at time of record creation
        let field_snapshot = self.schema_snapshot
            .fields
            .iter()
            .find(|f| f.field_id == field_id)
            .ok_or(RecordError::FieldNotInSnapshot)?;
        
        // Return the value as stored (original type)
        Ok(DisplayValue {
            value: cell_entry.value.clone(),
            field_config_version: field_snapshot.config_version,
            created_at: metadata.created_at,
            last_modified: metadata.updated_at,
        })
    }
}

// Usage
let record = db.get_record(record_id).await?;
let cell = record.get_cell(&field_id)?;
println!("Value: {} (version: {})", cell.value, cell.field_config_version);
// Output: Value: hello (version: 1)
```

#### Option B: Migrate On-Read (Automatic Conversion)

```rust
impl Record {
    pub fn get_cell_coerced(&self, field_id: &FieldId, registry: &TypeRegistry) -> Result<Value> {
        // 1. Get cell entry
        let cell_entry = self.cells
            .iter()
            .find(|c| c.field_id == field_id)
            .ok_or(RecordError::CellNotFound)?;
        
        // 2. Get original field config from snapshot
        let field_snapshot = self.schema_snapshot
            .fields
            .iter()
            .find(|f| f.field_id == field_id)
            .ok_or(RecordError::FieldNotInSnapshot)?;
        
        let old_config = self.schema_snapshot
            .get_field_config(field_snapshot.config_version)
            .await?;
        
        // 3. Get current field config
        let current_field = registry.get_field(field_id).await?;
        
        // 4. If they match, no conversion needed
        if field_snapshot.config_version == current_field.config_version {
            return Ok(cell_entry.value.clone());
        }
        
        // 5. Otherwise, transform
        println!("⚠️ On-read coercion: v{} → v{}", 
            field_snapshot.config_version,
            current_field.config_version
        );
        
        registry.transform_value(
            &cell_entry.value,
            &old_config,
            &current_field.config,
        )
    }
}

// Usage
let record = db.get_record(record_id).await?;
let value = record.get_cell_coerced(&field_id, &type_registry).await?;
// If field changed from Text to Number: "25" → 25
```

---

### Scenario 2: Detecting Schema Drift

**Situation:** You want to know "has this record's schema become outdated?"

```rust
impl Record {
    pub fn detect_schema_drift(&self, current_fields: &[Field]) -> Result<SchemaDrift> {
        let mut drifts = Vec::new();
        
        for field_snapshot in &self.schema_snapshot.fields {
            let current_field = current_fields
                .iter()
                .find(|f| f.id == Some(field_snapshot.field_id.clone()))
                .ok_or(RecordError::FieldDeleted)?;
            
            // Check if version changed
            if field_snapshot.config_version != current_field.config_version {
                drifts.push(FieldDrift {
                    field_id: field_snapshot.field_id.clone(),
                    old_version: field_snapshot.config_version,
                    new_version: current_field.config_version,
                    status: if drifts.is_empty() {
                        DriftStatus::NeedsMigration
                    } else {
                        DriftStatus::Drifted
                    },
                });
            }
            
            // Check if config hash changed
            let current_config_hash = sha256(&serde_json::to_string(&current_field.config)?);
            if field_snapshot.config_hash != current_config_hash {
                drifts.push(FieldDrift {
                    field_id: field_snapshot.field_id.clone(),
                    status: DriftStatus::ConfigChanged,
                });
            }
        }
        
        Ok(SchemaDrift {
            is_drifted: !drifts.is_empty(),
            record_schema_version: self.schema_version_id,
            current_schema_version: current_fields[0].table.latest_schema_version, // simplified
            drifts,
        })
    }
}

pub enum DriftStatus {
    NeedsMigration,   // Version is behind
    Drifted,          // Config hash changed
    ConfigChanged,    // But same version (?)
}

// Usage
let record = db.get_record(record_id).await?;
let drift = record.detect_schema_drift(&fields)?;

if drift.is_drifted {
    println!("⚠️ Record is drifted!");
    for drift in drift.drifts {
        println!("  Field {}: v{} → v{}", 
            drift.field_id, 
            drift.old_version, 
            drift.new_version
        );
    }
    // Suggest: run migration job
}
```

---

### Scenario 3: Creating a Migration Job

**Situation:** User changed field type from Text to Number, now you need to migrate all records

```rust
pub struct MigrationJobBuilder {
    field_id: FieldId,
    old_version: u32,
    new_version: u32,
    strategy: MigrationStrategy,
}

impl MigrationJobBuilder {
    pub async fn create_job(&self) -> Result<MigrationJob> {
        // 1. Get all records that need migration
        // SELECT COUNT(*) FROM record 
        // WHERE schema_snapshot CONTAINS field with version < new_version
        let records_to_migrate = db.query(
            "SELECT * FROM record WHERE schema_snapshot->[0].config_version < $version",
            vec![("$version", self.new_version)]
        ).await?;
        
        let total_records = records_to_migrate.len();
        
        // 2. Create job
        let job = MigrationJob {
            id: MigrationJobId::new(),
            field_id: self.field_id.clone(),
            from_version: self.old_version,
            to_version: self.new_version,
            strategy: self.strategy.clone(),
            state: MigrationState::Pending,
            batch_size: 1000,
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
            estimated_records: total_records as u32,
        };
        
        // 3. Store job
        db.insert_migration_job(&job).await?;
        
        Ok(job)
    }
}

// Usage
let job = MigrationJobBuilder {
    field_id: field_id.clone(),
    old_version: 1,
    new_version: 2,
    strategy: MigrationStrategy::SafeWithDefault {
        default_value: "0".to_string(),
    },
}
.create_job()
.await?;

println!("Created migration job: {}", job.id);
println!("Records to migrate: {}", job.estimated_records);
```

---

### Scenario 4: Executing a Migration Batch

**Situation:** Migration job is running, process one batch of 1000 records

```rust
pub struct MigrationBatchExecutor;

impl MigrationBatchExecutor {
    pub async fn execute_batch(
        batch: &mut MigrationBatch,
        migration_job: &MigrationJob,
        type_registry: &TypeRegistry,
    ) -> Result<MigrationBatchResult> {
        // 1. Fetch all records in this batch
        let mut records = db.get_records(&batch.record_ids).await?;
        
        // 2. Get field info
        let field = type_registry.get_field(&migration_job.field_id).await?;
        let old_config = type_registry
            .get_field_config_version(&migration_job.field_id, migration_job.from_version)
            .await?;
        let new_config = &field.config;
        
        // 3. Process each record
        let mut results = Vec::new();
        let mut success_count = 0;
        let mut fail_count = 0;
        
        for record in &mut records {
            // Find this field's cell in the record
            let cell_entry = record.cells
                .iter()
                .find(|c| c.field_id == migration_job.field_id)
                .cloned();
            
            match cell_entry {
                Some(entry) => {
                    // Transform the value
                    match type_registry.transform_value(
                        &entry.value,
                        &old_config,
                        new_config,
                    ) {
                        Ok(new_value) => {
                            // Update the cell
                            if let Some(cell) = record.cells.iter_mut()
                                .find(|c| c.field_id == migration_job.field_id) {
                                cell.value = new_value.clone();
                            }
                            
                            // Update metadata
                            if let Some(meta) = record.cell_metadata.entries.iter_mut()
                                .find(|m| m.cell_id == /* need to map */) {
                                meta.updated_at = Datetime::now();
                            }
                            
                            // Update schema snapshot version
                            if let Some(snap_field) = record.schema_snapshot.fields.iter_mut()
                                .find(|f| f.field_id == migration_job.field_id) {
                                snap_field.config_version = migration_job.to_version;
                                snap_field.config_hash = sha256(&serde_json::to_string(new_config)?);
                            }
                            
                            results.push(MigrationResult {
                                record_id: record.id.clone().unwrap(),
                                success: true,
                                old_value: Some(entry.value.clone()),
                                new_value: Some(new_value),
                                error: None,
                                transformed_at: Datetime::now(),
                            });
                            success_count += 1;
                        }
                        Err(e) => {
                            fail_count += 1;
                            results.push(MigrationResult {
                                record_id: record.id.clone().unwrap(),
                                success: false,
                                old_value: Some(entry.value.clone()),
                                new_value: None,
                                error: Some(e.to_string()),
                                transformed_at: Datetime::now(),
                            });
                        }
                    }
                }
                None => {
                    // Field doesn't exist in this record (nullable?)
                    results.push(MigrationResult {
                        record_id: record.id.clone().unwrap(),
                        success: true, // No-op is success
                        old_value: None,
                        new_value: None,
                        error: None,
                        transformed_at: Datetime::now(),
                    });
                    success_count += 1;
                }
            }
        }
        
        // 4. Persist all updated records
        for record in records {
            db.update_record(&record).await?;
        }
        
        // 5. Update batch state
        batch.state = MigrationState::Completed {
            completed_at: Datetime::now(),
            records_affected: success_count,
            records_failed: fail_count,
        };
        batch.results = results;
        
        db.update_migration_batch(batch).await?;
        
        Ok(MigrationBatchResult {
            batch_number: batch.batch_number,
            success_count,
            fail_count,
            total: success_count + fail_count,
        })
    }
}

// Usage
let mut batch = db.get_next_migration_batch(&job.id).await?;
let result = MigrationBatchExecutor::execute_batch(
    &mut batch,
    &job,
    &type_registry,
).await?;

println!("✓ Migrated: {}", result.success_count);
println!("✗ Failed: {}", result.fail_count);
```

---

### Scenario 5: Querying by Schema Version

**Situation:** You want to find all records that are still on an old schema version

```rust
impl RecordRepository {
    pub async fn find_records_by_schema_version(
        table_id: &TableId,
        schema_version: u32,
    ) -> Result<Vec<Record>> {
        db.query(
            "SELECT * FROM record 
             WHERE table = $table_id 
             AND schema_version_id = $version
             ORDER BY updated_at DESC",
            vec![
                ("$table_id", table_id.to_string()),
                ("$version", schema_version.to_string()),
            ]
        ).await
    }
    
    pub async fn find_drifted_records(
        table_id: &TableId,
        current_schema_version: u32,
    ) -> Result<Vec<Record>> {
        // Find records that are NOT on current schema
        db.query(
            "SELECT * FROM record 
             WHERE table = $table_id 
             AND schema_version_id < $current_version",
            vec![
                ("$table_id", table_id.to_string()),
                ("$current_version", current_schema_version.to_string()),
            ]
        ).await
    }
    
    pub async fn find_records_needing_migration(
        field_id: &FieldId,
        target_config_version: u32,
    ) -> Result<Vec<Record>> {
        // Find records where this field is behind
        db.query(
            "SELECT * FROM record 
             WHERE schema_snapshot[*].field_id = $field_id 
             AND schema_snapshot[*].config_version < $target_version",
            vec![
                ("$field_id", field_id.to_string()),
                ("$target_version", target_config_version.to_string()),
            ]
        ).await
    }
}

// Usage
let old_records = RecordRepository::find_records_by_schema_version(&table_id, 1).await?;
println!("Records on schema v1: {}", old_records.len());

let drifted = RecordRepository::find_drifted_records(&table_id, 3).await?;
println!("Drifted records: {}", drifted.len());
```

---

### Scenario 6: Rolling Back a Migration

**Situation:** Migration failed, need to rollback

```rust
pub struct MigrationRollback;

impl MigrationRollback {
    pub async fn rollback_batch(
        batch: &MigrationBatch,
        migration_job: &MigrationJob,
    ) -> Result<()> {
        // 1. For each failed transformation, restore old value
        for result in &batch.results {
            if result.success && result.old_value.is_some() {
                // Get the record
                let mut record = db.get_record(&result.record_id).await?;
                
                // Restore old value
                if let Some(cell) = record.cells.iter_mut()
                    .find(|c| c.field_id == migration_job.field_id) {
                    cell.value = result.old_value.clone().unwrap();
                }
                
                // Restore old schema version in snapshot
                if let Some(snap_field) = record.schema_snapshot.fields.iter_mut()
                    .find(|f| f.field_id == migration_job.field_id) {
                    snap_field.config_version = migration_job.from_version;
                }
                
                // Save
                db.update_record(&record).await?;
            }
        }
        
        // 2. Mark batch as rolled back
        db.query(
            "UPDATE migration_batch SET state = 'rolled_back' WHERE id = $batch_id",
            vec![("$batch_id", batch.id.to_string())]
        ).await?;
        
        println!("✓ Rolled back batch {}", batch.batch_number);
        Ok(())
    }
    
    pub async fn rollback_entire_job(job_id: &MigrationJobId) -> Result<()> {
        // Get all batches for this job
        let batches = db.query(
            "SELECT * FROM migration_batch WHERE job_id = $job_id",
            vec![("$job_id", job_id.to_string())]
        ).await?;
        
        // Rollback each batch in reverse order
        for batch in batches.iter().rev() {
            Self::rollback_batch(batch, &/* get job */).await?;
        }
        
        // Mark job as rolled back
        db.query(
            "UPDATE migration_job SET state = 'rolled_back' WHERE id = $job_id",
            vec![("$job_id", job_id.to_string())]
        ).await?;
        
        println!("✓ Rolled back entire migration job {}", job_id);
        Ok(())
    }
}

// Usage
MigrationRollback::rollback_entire_job(&job_id).await?;
```

---

### Scenario 7: Monitoring Migration Progress

**Situation:** Long-running migration, check progress

```rust
pub struct MigrationMonitor;

impl MigrationMonitor {
    pub async fn get_migration_status(job_id: &MigrationJobId) -> Result<MigrationStatus> {
        let job = db.get_migration_job(job_id).await?;
        
        let batches = db.query(
            "SELECT * FROM migration_batch WHERE job_id = $job_id",
            vec![("$job_id", job_id.to_string())]
        ).await?;
        
        let completed_batches = batches
            .iter()
            .filter(|b| matches!(b.state, MigrationState::Completed { .. }))
            .count();
        
        let failed_batches = batches
            .iter()
            .filter(|b| matches!(b.state, MigrationState::Failed { .. }))
            .count();
        
        let total_migrated: u32 = batches
            .iter()
            .filter_map(|b| {
                if let MigrationState::Completed { 
                    records_affected, .. 
                } = b.state {
                    Some(records_affected)
                } else {
                    None
                }
            })
            .sum();
        
        let total_failed: u32 = batches
            .iter()
            .filter_map(|b| {
                if let MigrationState::Completed { 
                    records_failed, .. 
                } = b.state {
                    Some(records_failed)
                } else {
                    None
                }
            })
            .sum();
        
        let progress_pct = if job.estimated_records > 0 {
            (total_migrated as f32 / job.estimated_records as f32) * 100.0
        } else {
            0.0
        };
        
        Ok(MigrationStatus {
            job_id: job_id.clone(),
            state: job.state.clone(),
            total_batches: batches.len(),
            completed_batches,
            failed_batches,
            total_records_to_migrate: job.estimated_records,
            total_migrated,
            total_failed,
            progress_pct,
            eta: calculate_eta(job.created_at, progress_pct),
        })
    }
    
    pub async fn print_migration_report(job_id: &MigrationJobId) -> Result<()> {
        let status = Self::get_migration_status(job_id).await?;
        
        println!("\n╔══════════════════════════════════════╗");
        println!("║  Migration Job: {}                   ║", status.job_id);
        println!("╚══════════════════════════════════════╝");
        println!("State:                  {}", status.state);
        println!("Progress:               {:.1}% ({}/{})", 
            status.progress_pct,
            status.total_migrated,
            status.total_records_to_migrate
        );
        println!("Batches:                {}/{}", 
            status.completed_batches,
            status.total_batches
        );
        println!("Failed:                 {}", status.total_failed);
        println!("ETA:                    {}", status.eta);
        println!();
        
        Ok(())
    }
}

// Usage (run in a loop)
loop {
    MigrationMonitor::print_migration_report(&job_id).await?;
    tokio::time::sleep(Duration::from_secs(5)).await;
}

// Output:
// ╔══════════════════════════════════════╗
// ║  Migration Job: migration|abc123     ║
// ╚══════════════════════════════════════╝
// State:                  InProgress
// Progress:               35.2% (3520/10000)
// Batches:                3/10
// Failed:                 0
// ETA:                    ~2 minutes 30 seconds
```

---

## Quick Reference: When to Use What

### Use `schema_snapshot` when:
✓ Reading a record (to know original types)
✓ Detecting schema drift
✓ Finding which records need migration
✓ Displaying historical field types
✓ Auditing what config a record was created with

### Use `config_version` when:
✓ Comparing old version to new version
✓ Creating migration jobs
✓ Checking if transformation is needed
✓ Building migration paths

### Use `FieldConfigVersion` table when:
✓ Looking up historical configs
✓ Understanding change history
✓ Auditing what changed and when
✓ Reverting to old config

### Use `MigrationJob` when:
✓ Bulk transforming records
✓ Tracking progress
✓ Rolling back changes
✓ Handling failures

---

## Complete Example: Text → Number Migration

```rust
// Step 1: User changes field type
let field_patch = FieldPatch {
    config: Some(FieldConfig::Number(NumberConfig::Number { default: None })),
    ..Default::default()
};
let old_field = db.get_field(&field_id).await?;
let version_entry = old_field.update(field_patch).await?;  // Now v2

println!("✓ Field updated to v2");
println!("  Migration strategy: {:?}", version_entry.migration_strategy);
// Output: Migration strategy: RiskyWithBackfill { transformation: "try_parse()" }

// Step 2: Detect what needs migration
let drifted = RecordRepository::find_records_needing_migration(&field_id, 2).await?;
println!("⚠️  {} records need migration", drifted.len());

// Step 3: Create migration job
let job = MigrationJobBuilder {
    field_id: field_id.clone(),
    old_version: 1,
    new_version: 2,
    strategy: MigrationStrategy::RiskyWithBackfill {
        default_value: "0".to_string(),
    },
}
.create_job()
.await?;

println!("Created migration job: {}", job.id);

// Step 4: Execute migration in batches
loop {
    if let Ok(mut batch) = db.get_next_migration_batch(&job.id).await {
        let result = MigrationBatchExecutor::execute_batch(
            &mut batch,
            &job,
            &type_registry,
        ).await?;
        
        println!("Batch {}: ✓{} ✗{}", 
            batch.batch_number, 
            result.success_count, 
            result.fail_count
        );
        
        if batch.batch_number == 10 { break; } // Or while more batches exist
    } else {
        break;
    }
}

// Step 5: Monitor
MigrationMonitor::print_migration_report(&job.id).await?;

// Step 6: All done!
println!("✓ Migration complete!");
```

---

## Troubleshooting

### Q: Record reads but shows wrong type
```rust
// This is EXPECTED if record hasn't been migrated yet
// Use get_cell_coerced() for automatic conversion
let value = record.get_cell_coerced(&field_id, &registry).await?;
```

### Q: Migration stopped halfway
```rust
// Check what failed
let batches = db.query(
    "SELECT * FROM migration_batch WHERE job_id = $id AND state = 'failed'",
    ...
).await?;

for batch in batches {
    for result in &batch.results {
        if !result.success {
            println!("Record {}: {}", result.record_id, result.error);
        }
    }
}
```

### Q: Need to rollback migration
```rust
// One command!
MigrationRollback::rollback_entire_job(&job_id).await?;
```

### Q: Which records are on which schema?
```rust
let v1_records = RecordRepository::find_records_by_schema_version(&table_id, 1).await?;
let v2_records = RecordRepository::find_records_by_schema_version(&table_id, 2).await?;
let v3_records = RecordRepository::find_records_by_schema_version(&table_id, 3).await?;

println!("Schema v1: {}", v1_records.len());
println!("Schema v2: {}", v2_records.len());
println!("Schema v3: {}", v3_records.len());
```

