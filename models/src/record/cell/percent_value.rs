use ordered_float::OrderedFloat;

use crate::{
    CurrencyValue, Value, ValueError, ValueType,
    kinds::{FieldConfig, NumberConfig},
    prelude::*,
};

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct PercentValue {
    pub value: i32,
}

impl ValueType<i32> for PercentValue {
    fn verify(&self, config: crate::kinds::FieldConfig) -> Result<(), super::ValueError> {
        if let FieldConfig::Number(NumberConfig::Percent { .. }) = config {
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
                    Value::SingleLine(crate::SingleLineValue { value: format!("{} %", self.value) })
                };
                LongText { rich_text } => {
                    Value::LongText(Box::new(crate::LongTextValue { value: format!("{} %", self.value)}))
                }
            };
            Number {
                Number { default } => {
                    Value::Number(crate::NumberValue { value: self.value as isize })
                };
                Decimal { precision, default } => {
                    Value::Decimal(crate::DecimalValue { value: crate::OrderedFloatIThink(OrderedFloat::from( self.value as f64)) })
                };
                Currency { currency, precision } => {
                    Value::Currency(CurrencyValue { value: crate::OrderedFloatIThink(OrderedFloat::from( self.value as f64)) })
                }
            };
        })
    }

    fn value(&self) -> &i32 {
        &self.value
    }
}

impl PercentValue {
    pub fn new(value: i32) -> Self {
        PercentValue { value }
    }
}
