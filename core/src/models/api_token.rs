use crate::models::ids::*;
use serde::{Deserialize, Serialize};
use surrealdb::types::Datetime;
use surrealdb::types::SurrealValue;

#[derive(SurrealValue, Debug, Clone, Serialize, Deserialize)]
pub struct ApiToken {
    pub id: Option<ApiTokenId>,
    pub user: UserId,
    pub token: String,
    pub created_at: Option<Datetime>,
    pub expires_at: Option<Datetime>,
    pub is_deleted: bool,
}

pub struct InsertApiToken {
    pub user: UserId,
    pub name: String,
    pub token: String,
    pub expires_at: Option<Datetime>,
}

impl ApiToken {
    pub fn from_insert(insert: InsertApiToken) -> Self {
        ApiToken {
            id: None,
            user: insert.user,
            token: insert.token,
            created_at: None,
            expires_at: insert.expires_at,
            is_deleted: false,
        }
    }
}
