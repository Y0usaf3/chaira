use crate::prelude::*;
pub mod kinds;
pub mod migration;
pub use self::kinds::*;

/// ['src/core/models/field.md']
#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct Field {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<FieldId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<Datetime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub config: FieldConfig,
    pub is_primary: bool,
    pub is_nullable: bool,
    pub is_unique: bool,
    pub name: String,
    pub order: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub fn apply_patch(&mut self, patch: FieldPatch) {
        if let Some(name) = patch.name {
            self.name = name;
        }
        if let Some(description) = patch.description {
            self.description = Some(description);
        }
        if let Some(is_primary) = patch.is_primary {
            self.is_primary = is_primary;
        }
        if let Some(is_nullable) = patch.is_nullable {
            self.is_nullable = is_nullable;
        }
        if let Some(is_unique) = patch.is_unique {
            self.is_unique = is_unique;
        }
        if let Some(order) = patch.order {
            self.order = order;
        }
        if let Some(config) = patch.config {
            self.config = config;
        }

        self.updated_at = Some(Datetime::now());
    }
}
