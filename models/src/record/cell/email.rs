use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use validator::ValidateEmail;

use super::long_text_value::LongTextValue;
use super::single_line_value::SingleLineValue;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Email {
    pub(super) value: String,
}

impl ValueType<str> for Email {
    fn verify(&self, config: FieldConfig) -> Result<(), ValueError> {
        if let FieldConfig::Text(TextConfig::Email) = config {
            if self.value.validate_email() {
                Ok(())
            } else {
                Err(ValueError::InvalidEmail(self.value.clone()))
            }
        } else {
            Err(ValueError::WrongType(format!("{:?}", config)))
        }
    }
    fn convert_to(&self, target_config: &FieldConfig) -> Result<Value, ValueError>
    where
        Self: Sized,
    {
        try_convert!( target_config {
            Text {
                SingleLine { default, max_length } => {
                    Value::SingleLine(SingleLineValue { value: self.value.chars().take(*max_length as usize).collect() })
                };
                LongText { rich_text } => {
                    Value::LongText(Box::new(LongTextValue { value: self.value.clone() }))
                }
            };
        })
    }

    fn value(&self) -> &str {
        &self.value
    }
}

impl Email {
    pub fn new(value: String) -> Result<Self, super::ValueError> {
        if value.validate_email() {
            Ok(Self {
                value: value.trim().to_lowercase(),
            })
        } else {
            Err(super::ValueError::InvalidEmail(value))
        }
    }
}
