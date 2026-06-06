use super::value::Value;
use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellValue {
    pub id: CellId,
    pub created_at: Datetime,
    pub updated_at: Datetime,
    pub value: Value,
}

impl CellValue {
    pub fn new(value: Value) -> Self {
        use surrealdb_types::RecordId as Thing;
        Self {
            id: CellId(Thing {
                table: "cell".into(),
                key: Uuid::new_v4().to_string().into(),
            }),
            created_at: Datetime::now(),
            updated_at: Datetime::now(),
            value,
        }
    }
}
