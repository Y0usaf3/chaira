use ordered_float::OrderedFloat;

use crate::{
    LongTextValue, OrderedFloatIThink, SingleLineValue, Value, ValueError, ValueType,
    kinds::FieldConfig, prelude::*,
};

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct RatingValue {
    pub value: u8,
}

impl ValueType<u8> for RatingValue {
    fn verify(&self, config: crate::kinds::FieldConfig) -> Result<(), super::ValueError> {
        if let FieldConfig::Number(crate::kinds::NumberConfig::Rating { max, .. }) = config {
            if self.value > max as u8 {
                Err(ValueError::BiggerThanMax)
            } else {
                Ok(())
            }
        } else {
            Err(super::ValueError::WrongType(format!("{config:?}")))
        }
    }

    fn convert_to(&self, target_config: &FieldConfig) -> Result<super::Value, ValueError>
    where
        Self: Sized,
    {
        try_convert!(target_config {
            Text {
                SingleLine { default, max_length } => {
                    Value::SingleLine(SingleLineValue { value: self.value.to_string().chars().take(*max_length as usize).collect() })
                };
                LongText { rich_text } => {
                    Value::LongText(Box::new(LongTextValue { value: self.value.to_string() }))
                }
            };
            Number {
                Number { default } => {
                    Value::Number(crate::NumberValue { value: self.value as isize })
                };
                Decimal { precision, default } => {
                    Value::Decimal(crate::DecimalValue { value: crate::OrderedFloatIThink(OrderedFloat::from(self.value as f64)) })
                };
                Currency { currency, precision } => {
                    Value::Currency(crate::CurrencyValue { value: OrderedFloatIThink(OrderedFloat::from(self.value as f64)) })
                };
                Percent { show_bar, precision } => {
                    Value::Percent(crate::PercentValue { value: self.value as i32 })
                }
            };
        })
    }

    fn value(&self) -> &u8 {
        &self.value
    }
}

impl RatingValue {
    pub fn new(value: Option<u8>, max: u8) -> Result<Self, super::ValueError> {
        let ratings = value.unwrap_or(0);
        if ratings > max {
            return Err(super::ValueError::RatingExceedsMax {
                value: ratings,
                max,
            });
        };
        Ok(Self { value: ratings })
    }
}
