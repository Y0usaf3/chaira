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

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
