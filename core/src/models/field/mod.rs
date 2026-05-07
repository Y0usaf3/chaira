pub use crate::models::field::kinds::*;
use crate::models::ids::*;
use surrealdb::types::{Datetime, SurrealValue};

pub mod kinds;
pub mod migration;

/// ['src/core/models/field.md']
#[derive(SurrealValue, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub id: Option<FieldId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub config: FieldConfig,
    pub is_primary: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub name: String,
    pub order: u32,
    pub description: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InsertField {
    pub name: String,
    pub description: Option<String>,
    pub is_primary: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub order: u32,
    pub config: FieldConfig,
}

impl InsertField {
    pub fn new(
        name: String,
        config: FieldConfig,
        is_primary: bool,
        is_nullable: bool,
        is_unique: bool,
    ) -> Self {
        Self {
            name,
            config,
            is_primary,
            is_nullable,
            is_unique,
            description: None,
            order: 0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FieldPatch {
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_primary: Option<bool>,
    pub is_nullable: Option<bool>,
    pub is_unique: Option<bool>,
    pub order: Option<u32>,
    pub config: Option<FieldConfig>,
}

impl Field {
    pub fn from_insert(insert: InsertField) -> Self {
        Field {
            id: None,
            created_at: None,
            updated_at: None,
            is_deleted: false,
            config: insert.config,
            is_primary: insert.is_primary,
            is_nullable: insert.is_nullable,
            is_unique: insert.is_unique,
            name: insert.name,
            order: insert.order,
            description: insert.description,
        }
    }
}
