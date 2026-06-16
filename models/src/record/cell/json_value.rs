use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct JsonValue {
    pub value: String,
}

impl JsonValue {
    pub fn new(value: String) -> Result<Self, super::ValueError> {
        serde_json::from_str::<serde_json::Value>(&value)
            .map_err(|e| super::ValueError::InvalidJson(e.to_string()))?;
        Ok(JsonValue { value })
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }
}
