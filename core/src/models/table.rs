use crate::models::ids::*;
use surrealdb::types::{Datetime, SurrealValue};

// A Table represents a logical container within a Base.
//
// Each Table belongs to exactly one Base.
// Only authorized users of the parent Base (typically the workspace owner or admins)
// may modify or soft-delete a Table.

#[derive(Debug, Clone, PartialEq, SurrealValue, serde::Serialize, serde::Deserialize)]
pub struct Table {
    pub id: Option<TableId>,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub is_deleted: bool,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InsertTable {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TablePatch {
    pub name: Option<String>,
    pub is_deleted: Option<bool>,
}

impl Table {
    pub fn from_insert(insert: InsertTable) -> Self {
        Table {
            id: None,
            created_at: Datetime::from(chrono::Utc::now()),
            updated_at: Datetime::from(chrono::Utc::now()),
            is_deleted: false,
            name: insert.name,
        }
    }
}
