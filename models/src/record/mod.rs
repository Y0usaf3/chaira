use crate::prelude::*;
use std::collections::HashMap;
use surrealdb::types::ToSql;

const VERSION: u8 = 1;

pub mod cell;
pub use self::cell::*;

#[derive(Debug, Clone, SurrealValue, Deserialize, Serialize)]
pub struct Record {
    pub id: Option<RecordId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub cells: HashMap<String, Value>,
    pub cell_metadata: HashMap<String, CellMetadata>,
    pub version: u8,
    pub table: TableId,
}

#[derive(Debug, Clone, SurrealValue, Deserialize, Serialize)]
pub struct CellMetadata {
    pub created_at: Datetime,
    pub updated_at: Datetime,
}

impl Default for CellMetadata {
    fn default() -> Self {
        let now = Datetime::now();
        Self {
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
pub struct InsertRecord {
    pub table: TableId,
    pub cells: HashMap<String, Value>,
}

impl InsertRecord {
    pub fn new(table: TableId, cells: HashMap<String, Value>) -> Self {
        Self { table, cells }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecordPatch {
    pub changed_cells: Option<Vec<(String, Value)>>,
}

impl RecordPatch {
    pub fn new(changed_cells: Option<Vec<(String, Value)>>) -> Self {
        Self { changed_cells }
    }
}

impl Record {
    pub fn from_insert(insert: InsertRecord) -> Self {
        let cells = insert.cells.clone().into_iter().collect();
        let cell_metadata = insert
            .cells
            .into_iter()
            .map(|key| (key.0, CellMetadata::default()))
            .collect();

        Record {
            id: None,
            created_at: None,
            updated_at: None,
            is_deleted: false,
            cells,
            cell_metadata,
            version: VERSION,
            table: insert.table,
        }
    }

    pub fn upsert_cell(&mut self, field_id: String, value: Value) {
        self.cells.insert(field_id, value);
    }

    pub fn delete_cell(&mut self, field_id: FieldId) {
        let field_id = field_id.0.key.to_sql();
        self.cells.remove(&field_id);
    }

    pub fn apply_patch(&mut self, patch: RecordPatch) {
        if let Some(changes) = patch.changed_cells {
            for (cell_name, new_value) in changes {
                self.upsert_cell(cell_name, new_value);
            }
        }
    }
}
