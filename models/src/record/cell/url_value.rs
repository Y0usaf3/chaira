use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use serde::de::value;
use validator::ValidateUrl;

use super::long_text_value::LongTextValue;
use super::single_line_value::SingleLineValue;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UrlValue {
    pub(super) value: String,
}

impl ValueType<str> for UrlValue {
    fn verify(&self, config: FieldConfig) -> Result<(), ValueError> {
        if let FieldConfig::Text(TextConfig::URL) = config {
            if self.value.validate_url() {
                Ok(())
            } else {
                Err(ValueError::InvalidUrl(self.value.clone()))
            }
        } else {
            Err(ValueError::WrongType(format!("{config:?}")))
        }
    }

    fn convert_to(&self, target_config: &FieldConfig) -> Result<Value, ValueError>
    where
        Self: Sized,
    {
        try_convert!(target_config {
            Text {
                SingleLine { default, max_length } => {
                    Value::SingleLine(SingleLineValue { value: self.value.clone().chars().take(*max_length as usize).collect() })
                };
                LongText { rich_text } => {
                    let value = if *rich_text {
                        self.value.clone().replace(['*', '_', '#', '`', '[', ']'], "")
                    } else {
                        self.value.clone()
                    };

                    Value::LongText(Box::new(LongTextValue { value }))
                }
            };
        })
    }

    fn value(&self) -> &str {
        &self.value
    }
}

impl UrlValue {
    pub fn new(value: String) -> Result<Self, super::ValueError> {
        let a = value.clone();
        let valuee = value
            .trim()
            .split(' ')
            .find(|v| v.validate_url())
            .ok_or(ValueError::InvalidUrl(a))?;
        Ok(Self {
            value: valuee.to_string(),
        })
    }
}
