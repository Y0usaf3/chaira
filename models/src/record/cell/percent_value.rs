use crate::{
    Value, ValueError, ValueType,
    kinds::{FieldConfig, NumberConfig},
    prelude::*,
};

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
