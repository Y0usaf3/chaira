use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct ModifiedTimeValue {
    pub value: Datetime,
}

impl ModifiedTimeValue {
    pub fn new(value: Datetime) -> Self {
        ModifiedTimeValue { value }
    }
    pub fn value(&self) -> &Datetime {
        &self.value
    }
}
