use ordered_float::OrderedFloat;

use crate::kinds::FieldConfig;
use crate::{DecimalValue, prelude::*};
use crate::{ValueError, ValueType};

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NumberValue {
    pub value: isize,
}

impl ValueType<isize> for NumberValue {
    fn verify(&self, config: crate::kinds::FieldConfig) -> Result<(), super::ValueError> {
        if let FieldConfig::Number(crate::kinds::NumberConfig::Number { .. }) = config {
            Ok(())
        } else {
            Err(ValueError::WrongType(format!("{config:?}")))
        }
    }
    fn convert_to(&self, target_config: &FieldConfig) -> Result<super::Value, ValueError>
    where
        Self: Sized,
    {
        try_convert!(target_config {
            Number {
                Decimal { default, precision } => {
                    crate::Value::Decimal(DecimalValue {
                        value: crate::OrderedFloatIThink(OrderedFloat::from(self.value as f64))
                    })
                }
            };
        })
    }

    fn value(&self) -> &isize {
        &self.value
    }
}

impl NumberValue {
    pub fn new(value: Option<isize>, default: Option<isize>) -> Result<Self, super::ValueError> {
        if value.is_none() && default.is_none() {
            return Err(super::ValueError::MissingValue);
        };
        if let Some(v) = value {
            Ok(NumberValue { value: v })
        } else {
            Ok(NumberValue {
                value: default.unwrap_or(0),
            })
        }
    }
}
