use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DateValue {
    value: Datetime,
}

impl DateValue {
    pub fn new(value: Datetime) -> Self {
        DateValue { value }
    }

    pub fn value(&self) -> &Datetime {
        &self.value
    }
}
