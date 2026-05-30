use crate::prelude::*;
use surrealdb_types::{record_id::RecordId as Thing, uuid};

macro_rules! define_ids {
    ($($name:ident),*) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq, Hash, SurrealValue, Serialize, Deserialize)]
            pub struct $name(pub  Thing);
        )*
    };
}

// Now you can define all of them at once :3
define_ids!(
    BaseId, TableId, UserId, CellId, RowId, RecordId, FieldId, RelationId, IdentityId, ViewId
);

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    pub fn new() -> Self {
        SessionId(format!("session:{}", uuid::Uuid::new_v4()))
    }
}
