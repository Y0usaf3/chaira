use crate::kinds::Prefix;
use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AutoNumberValue {
    value: usize,
    prefix: Prefix,
    formatted: String,
}

impl AutoNumberValue {
    pub fn new(value: usize, prefix: Prefix) -> Self {
        let prefix_str = match prefix {
            Prefix::Dot => '•',
            Prefix::Star => '*',
        };

        let formatted = format!("{}{}", prefix_str, value);
        AutoNumberValue {
            value,
            prefix,
            formatted,
        }
    }

    pub fn formatted(&self) -> &str {
        self.formatted.as_str()
    }

    pub fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    pub fn value(&self) -> &usize {
        &self.value
    }
}
