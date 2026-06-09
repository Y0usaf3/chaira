use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CreatedAtValue {
    pub value: Datetime,
}

impl CreatedAtValue {
    pub fn new(value: Datetime) -> Self {
        CreatedAtValue { value }
    }
    pub fn value(&self) -> &Datetime {
        &self.value
    }
}
