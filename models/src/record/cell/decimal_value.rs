use super::ordered_float_i_think::OrderedFloatIThink;
use crate::prelude::*;
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DecimalValue {
    pub value: OrderedFloatIThink,
}

impl DecimalValue {
    pub fn new(value: Option<f64>, default: Option<f64>) -> Result<Self, super::ValueError> {
        if value.is_none() && default.is_none() {
            return Err(super::ValueError::MissingValue);
        };
        if let Some(v) = value {
            Ok(DecimalValue {
                value: OrderedFloatIThink(OrderedFloat::from(v)),
            })
        } else {
            Ok(DecimalValue {
                value: OrderedFloatIThink(OrderedFloat::from(default.unwrap_or(0.0))),
            })
        }
    }

    pub fn value(&self) -> &OrderedFloat<f64> {
        &self.value.0
    }
}
