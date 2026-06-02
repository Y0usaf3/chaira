use crate::prelude::*;
use std::collections::HashMap;
pub mod cell;
pub use self::cell::*;

// TODO: Verify trait to automaticly verify if the record is FieldConfig compliant
// TODO: rework the whole damn record thing
// TODO: snapshots and stuff

#[derive(Debug, SurrealValue, Deserialize, Serialize)]
pub struct Record {
    pub id: Option<RecordId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub cells: HashMap<String, Value>,
    pub cell_metadata: Vec<CellMetadata>,
    pub schema_version: u32,
    pub schema_snapshots: SchemaSnapshots,
}

#[derive(Debug, SurrealValue, Deserialize, Serialize)]
pub struct CellMetadata {
    id: CellId,
    updated_at: Datetime,
    created_at: Datetime,
}

#[derive(Debug, SurrealValue, Deserialize, Serialize)]
pub struct SchemaSnapshots {
    version: u32,
    created_at: Datetime,
    fields: Vec<FieldSnapshot>,
    hash: String,
}

#[derive(Debug, SurrealValue, Deserialize, Serialize)]
pub struct FieldSnapshot {
    field_id: FieldId,
    name: String,
    config_version: u32,
    config_hash: String,
}

//
// #[derive(Debug, Clone, PartialEq, Eq, SurrealValue, serde::Serialize, serde::Deserialize)]
// pub struct Record {
//     pub id: Option<RecordId>,
//     pub created_at: Option<Datetime>,
//     pub updated_at: Option<Datetime>,
//     pub is_deleted: bool,
//     pub cells: HashMap<String, CellValue>, // K: FieldId
//     pub table: TableId,
// }
//
// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, SurrealValue)]
// pub struct InsertRecord {
//     pub table: TableId,
//     pub cells: HashMap<String, CellValue>,
// }
//
// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// pub struct RecordPatch {
//     pub changed_cells: Option<Vec<(String, CellValue)>>,
// }
//
// impl InsertRecord {
//     pub fn new(table: TableId, cells: HashMap<String, CellValue>) -> Self {
//         Self { table, cells }
//     }
// }
//
// impl RecordPatch {
//     pub fn new(changed_cells: Option<Vec<(String, CellValue)>>) -> Self {
//         Self { changed_cells }
//     }
// }
//
// impl Record {
//     pub fn from_insert(insert: InsertRecord) -> Self {
//         Record {
//             id: None,
//             created_at: None,
//             updated_at: None,
//             is_deleted: false,
//             cells: insert.cells,
//             table: insert.table,
//         }
//     }
// }
