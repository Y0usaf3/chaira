use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PercentValue {
    value: i32,
}

impl PercentValue {
    pub fn new(value: i32) -> Self {
        PercentValue { value }
    }
    pub fn value(&self) -> &i32 {
        &self.value
    }
}
