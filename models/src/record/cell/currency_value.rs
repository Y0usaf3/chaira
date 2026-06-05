use crate::{ValueType, kinds::FieldConfig, prelude::*};
use iso_currency::{Currency, CurrencySymbol};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CurrencyValue {
    value: OrderedFloat<f32>,
}

impl ValueType for CurrencyValue {
    fn verify(&self, config: crate::kinds::FieldConfig) -> Result<(), super::ValueError> {}
}

impl CurrencyValue {
    pub fn new(amount: OrderedFloat<f32>) -> Self {
        CurrencyValue { value: amount }
    }
}
