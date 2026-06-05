use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DurationValue {
    value: Duration,
}

impl DurationValue {
    pub fn new(value: Duration) -> Self {
        DurationValue { value }
    }

    pub fn value(&self) -> &Duration {
        &self.value
    }
}
