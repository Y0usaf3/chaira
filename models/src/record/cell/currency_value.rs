use crate::{
    LongTextValue, OrderedFloatIThink, Value, ValueError, ValueType, kinds::FieldConfig, prelude::*,
};
use ordered_float::OrderedFloat;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CurrencyValue {
    pub value: OrderedFloatIThink,
}

impl ValueType<OrderedFloatIThink> for CurrencyValue {
    fn verify(&self, config: crate::kinds::FieldConfig) -> Result<(), super::ValueError> {
        if let FieldConfig::Number(crate::kinds::NumberConfig::Currency { .. }) = config {
            Ok(())
        } else {
            Err(ValueError::WrongType(format!("{config:?}")))
        }
    }

    fn convert_to(&self, target_config: &FieldConfig) -> Result<super::Value, ValueError>
    where
        Self: Sized,
    {
        try_convert!(target_config  {
            Text {
                SingleLine { default, max_length } => {
                    Value::SingleLine(crate::SingleLineValue { value: self.value.0.to_string().chars().take(*max_length as usize).collect() })
                };
                LongText { rich_text } => {
                    Value::LongText(Box::new(LongTextValue { value: self.value.0.to_string() }) )
                }
            };
            Number {
                Number { default } => {
                    Value::Number(crate::NumberValue { value: self.value.0.round() as isize })
                };
                Decimal { precision, default } => {
                    Value::Decimal(crate::DecimalValue { value: self.value.clone() })
                };
                Percent { show_bar, precision } => {
                    Value::Percent(crate::PercentValue { value: self.value.0.round() as i32 })
                }
            };
        })
    }

    fn value(&self) -> &OrderedFloatIThink {
        &self.value
    }
}

impl CurrencyValue {
    pub fn new(amount: OrderedFloat<f64>) -> Self {
        CurrencyValue {
            value: OrderedFloatIThink(amount),
        }
    }
}
