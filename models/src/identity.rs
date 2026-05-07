use crate::prelude::*;

#[derive(SurrealValue)]
pub struct Identity {
    pub id: Option<IdentityId>,
    pub user: UserId,
    pub external_user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Datetime,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
}

pub struct InsertIdentity {
    pub user: UserId,
    pub external_user_id: String,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at: Datetime,
}

pub struct IdentityPatch {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<Datetime>,
    pub is_deleted: Option<bool>,
}

impl Identity {
    pub fn from_insert(insert: InsertIdentity) -> Self {
        Identity {
            id: None,
            user: insert.user,
            external_user_id: insert.external_user_id,
            access_token: insert.access_token,
            refresh_token: insert.refresh_token,
            expires_at: insert.expires_at,
            created_at: None,
            updated_at: None,
            is_deleted: false,
        }
    }
}
