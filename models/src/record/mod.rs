use surrealdb_types::ToSql;

use crate::prelude::*;
use std::collections::HashMap;

const VERSION: u8 = 1;

pub mod cell;
pub use self::cell::*;

#[derive(Debug, Clone, SurrealValue, Deserialize, Serialize)]
pub struct Record {
    pub id: Option<RecordId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub cells: HashMap<String, CellValue>,
    pub version: u8,
    pub table: TableId,
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
        let cells = insert
            .cells
            .into_iter()
            .map(|(key, value)| (key, CellValue::new(value)))
            .collect();

        Record {
            id: None,
            created_at: None,
            updated_at: None,
            is_deleted: false,
            cells,
            version: VERSION,
            table: insert.table,
        }
    }

    pub fn upsert_cell(&mut self, field_id: String, value: Value) {
        self.cells.insert(field_id, CellValue::new(value));
        self.version = self.version.saturating_add(1);
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
