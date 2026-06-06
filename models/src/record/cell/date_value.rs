use crate::{
    SingleLineValue, Value, ValueError, ValueType,
    kinds::{DatetimeConfig, FieldConfig},
    prelude::*,
};

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DateValue {
    pub value: Datetime,
}

impl ValueType<Datetime> for DateValue {
    fn verify(&self, config: crate::kinds::FieldConfig) -> Result<(), super::ValueError> {
        if let FieldConfig::Datetime(DatetimeConfig::Date {
            format,
            include_time,
        }) = config
        {
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
                    Value::SingleLine(SingleLineValue { value: self.value.to_string() })
                }
            };
        })
    }

    fn value(&self) -> &Datetime {
        &self.value
    }
}

impl DateValue {
    pub fn new(value: Datetime) -> Self {
        DateValue { value }
    }
}
