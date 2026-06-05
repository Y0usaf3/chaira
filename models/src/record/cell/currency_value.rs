use crate::prelude::*;
use iso_currency::CurrencySymbol;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CurrencyValue {
    value: i64,
    currency_symbole: String,
    formatted: String,
}

impl CurrencyValue {
    pub fn new(value: i64, currency_symbole: CurrencySymbol) -> Self {
        let formatted = format!("{} {}", value, &currency_symbole.symbol);
        CurrencyValue {
            value,
            currency_symbole: currency_symbole.to_string(),
            formatted,
        }
    }

    pub fn value_as_int(&self) -> &i64 {
        &self.value
    }

    pub fn value_as_str(&self) -> &str {
        &self.formatted
    }
}
