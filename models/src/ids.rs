use ::serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;
use surrealdb_types::record_id::RecordId as Thing;

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
    BaseId, TableId, UserId, CellId, RowId, RecordId, FieldId, RelationId, IdentityId, SessionId,
    ViewId
);
