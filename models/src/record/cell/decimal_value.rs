use super::ordered_float_i_think::OrderedFloatIThink;
use crate::{
    CurrencyValue, LongTextValue, NumberValue, Value, ValueType, kinds::FieldConfig, prelude::*,
};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DecimalValue {
    pub value: OrderedFloatIThink,
}

impl ValueType<OrderedFloat<f64>> for DecimalValue {
    fn verify(&self, config: crate::kinds::FieldConfig) -> Result<(), super::ValueError> {
        if let FieldConfig::Number(crate::kinds::NumberConfig::Decimal { .. }) = config {
            Ok(())
        } else {
            Err(super::ValueError::WrongType(format!("{config:?}")))
        }
    }

    fn convert_to(&self, target_config: &FieldConfig) -> Result<super::Value, super::ValueError>
    where
        Self: Sized,
    {
        try_convert!(target_config {
            Text {
                SingleLine { default, max_length } => {
                    crate::Value::SingleLine(crate::SingleLineValue { value: self.value.0.to_string().chars().take(*max_length as usize).collect() })
                };
                LongText { rich_text } => {
                    Value::LongText(Box::new(LongTextValue { value: self.value.0.to_string() }))
                }
            };
            Number {
                Number { default } => {
                    Value::Number(NumberValue { value: self.value.0.round() as isize })
                };
                Currency { currency, precision } => {
                    Value::Currency(CurrencyValue { value: self.value.clone() })
                };
                Percent { precision, show_bar } => {
                    Value::Percent(crate::PercentValue { value: self.value.0.round() as i32 })
                }
            };
        })
    }

    fn value(&self) -> &OrderedFloat<f64> {
        &self.value.0
    }
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
}
