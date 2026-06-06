use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use phonenumber::PhoneNumber;
use serde::de::value;
use std::str::FromStr;

use super::single_line_value::SingleLineValue;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PhoneValue {
    pub(super) value: String,
}

impl ValueType<str> for PhoneValue {
    fn verify(&self, config: FieldConfig) -> Result<(), ValueError> {
        if let FieldConfig::Text(TextConfig::Phone) = config {
            if PhoneNumber::from_str(&self.value).is_ok() {
                Ok(())
            } else {
                Err(ValueError::InvalidPhoneNumber(self.value.clone()))
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
                    Value::SingleLine(SingleLineValue { value: self.value.clone() })
                };
                LongText { rich_text } => {
                    Value::Phone(PhoneValue { value: self.value.clone() })
                }
            };
        })
    }

    fn value(&self) -> &str {
        &self.value
    }
}

impl PhoneValue {
    pub fn new(value: String, default_region: Option<&str>) -> Result<Self, super::ValueError> {
        let region = default_region.and_then(|r| r.parse().ok());
        let a = value.clone();
        let value = value
            .split(' ')
            .find(|v| PhoneNumber::from_str(v).is_ok())
            .ok_or(ValueError::InvalidPhoneNumber(a))?;

        match phonenumber::parse(region, value) {
            Ok(phone) => {
                if phonenumber::is_valid(&phone) {
                    let formatted = phone.format().mode(phonenumber::Mode::E164).to_string();
                    Ok(Self { value: formatted })
                } else {
                    Err(super::ValueError::InvalidPhoneNumber(value.to_string()))
                }
            }
            Err(_) => Err(super::ValueError::UnparseablePhoneNumber(value.to_string())),
        }
    }
}
