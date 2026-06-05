use std::str::FromStr;

use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use phonenumber::PhoneNumber;
use validator::ValidateEmail;
use validator::ValidateLength;
use validator::ValidateUrl;

use super::email::Email;
use super::max_text_length::MAX_TEXT_LENGHT;
use super::phone_value::PhoneValue;
use super::single_line_value::SingleLineValue;
use super::url_value::UrlValue;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LongTextValue {
    pub(super) value: String,
}

impl ValueType<str> for LongTextValue {
    fn verify(&self, config: FieldConfig) -> Result<(), ValueError> {
        if let FieldConfig::Text(TextConfig::LongText { rich_text }) = config {
            if !rich_text && self.value.contains(['*', '_', '#', '`', '[', ']']) {
                return Err(ValueError::UnallowedRichType);
            };
            Ok(())
        } else {
            Err(ValueError::WrongType(format!("{:?}", config)))
        }
    }

    fn convert_to(&self, target_config: &FieldConfig) -> Result<Value, ValueError>
    where
        Self: Sized,
    {
        try_convert!(target_config {
            Text {
                 SingleLine { max_length, default } => {
                    let value = self.value.clone().replace("\n", " ").replace("\r", " ");
                    Value::SingleLine(SingleLineValue { value: value.chars().take(*max_length as usize).collect() })
                };
                Email => {
                    let valid_email = self.value
                        .lines()
                        .map(|v| v.trim())
                        .find(|v| v.validate_email());
                    Value::Email(Email { value: valid_email.ok_or(ValueError::CantConvertTo("Email".to_string()))?.to_string() })
                };
                URL => {
                    let valid_url = self.value.trim().split(' ').find(|v| v.validate_url());
                    Value::URL(UrlValue { value: valid_url.ok_or(ValueError::CantConvertTo("URL".to_string()))?.to_string() })
                };
                Phone => {
                    let valid_phone = self.value.trim().split(' ').find(|v| PhoneNumber::from_str(v).is_ok());
                    Value::Phone(PhoneValue { value: PhoneNumber::from_str(
                        valid_phone
                            .ok_or(ValueError::CantConvertTo("Phone".to_string()))?)
                            .ok()
                            .ok_or(ValueError::CantConvertTo("Phone".to_string()))?
                            .format().mode(phonenumber::Mode::E164).to_string()
                    })
                }
            };
        })
    }

    fn value(&self) -> &str {
        &self.value
    }
}

impl LongTextValue {
    pub fn new(value: String, rich_text: bool) -> Result<Self, super::ValueError> {
        let processed = if rich_text {
            value.trim().to_string()
        } else {
            value
                .replace(['*', '_', '#', '`', '[', ']'], "")
                .trim()
                .to_string()
        };
        let text_lenght = processed.length().unwrap();
        if text_lenght > MAX_TEXT_LENGHT.into() {
            return Err(super::ValueError::TextTooBig(text_lenght));
        };
        Ok(Self { value: processed })
    }
}
