use crate::models::ids::*;
use surrealdb::types::{Datetime, SurrealValue};

// A Base represents a sub-entity within a Workspace.
//
// Each Base belongs to exactly one Workspace, identified by `workspace`.
// Only authorized users of the parent Workspace (typically the owner or admins)
// are allowed to modify or soft-delete a Base.

#[derive(SurrealValue, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Base {
    pub id: Option<BaseId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub owner: UserId,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InsertBase {
    pub name: String,
    pub owner: UserId,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BasePatch {
    pub is_deleted: Option<bool>,
    pub name: Option<String>,
}

impl Base {
    pub fn from_insert(insert: InsertBase) -> Self {
        Base {
            id: None,
            created_at: None,
            updated_at: None,
            is_deleted: false,
            name: insert.name,
            owner: insert.owner,
        }
    }
}
