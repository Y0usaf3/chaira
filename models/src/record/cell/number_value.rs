use crate::kinds::FieldConfig;
use crate::{CurrencyValue, DecimalValue, LongTextValue, PercentValue, Value, prelude::*};
use crate::{ValueError, ValueType};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
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
            Text {
                SingleLine { default, max_length } => {
                    Value::SingleLine(crate::SingleLineValue { value: self.value.to_string().chars().take(*max_length as usize).collect() })
                };
                LongText { rich_text } => {
                    Value::LongText(Box::new(LongTextValue { value: self.value.to_string() }))
                }
            };
            Number {
                Decimal { default, precision } => {
                    crate::Value::Decimal(DecimalValue {
                        value: crate::OrderedFloatIThink(OrderedFloat::from(self.value as f64))
                    })
                };
                Currency { currency, precision } => {
                    Value::Currency(CurrencyValue { value: crate::OrderedFloatIThink(OrderedFloat::from(self.value as f64)) })
                };
                Percent { precision, show_bar } => {
                    Value::Percent(PercentValue { value: self.value as i32 })
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
