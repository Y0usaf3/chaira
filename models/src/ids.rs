use crate::prelude::*;
use surrealdb_types::{record_id::RecordId as Thing, uuid};

macro_rules! define_ids {
    ($($name:ident),*) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq, Hash, SurrealValue, Serialize, Deserialize)]
#[surreal(crate = "::surrealdb_types")]
            pub struct $name(pub  Thing);
        )*
    };
}

// Now you can define all of them at once :3
define_ids!(
    BaseId, TableId, UserId, CellId, RowId, RecordId, FieldId, RelationId, IdentityId, ViewId
);

impl BaseId {
    pub fn parse(key: &str) -> Result<Self, String> {
        let rid = Thing::parse_simple(&format!("base:{key}")).map_err(|e| format!("{e:?}"))?;
        Ok(BaseId(rid))
    }
}

impl TableId {
    pub fn parse(key: &str) -> Result<Self, String> {
        let rid = Thing::parse_simple(&format!("table:{key}")).map_err(|e| format!("{e:?}"))?;
        Ok(TableId(rid))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionId(pub String);

impl SessionId {
    pub fn new() -> Self {
        SessionId(format!("session:{}", uuid::Uuid::new_v4()))
    }
}
