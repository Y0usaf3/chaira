use std::str::FromStr;

use crate::NumberValue;
use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use ordered_float::OrderedFloat;
use phonenumber::PhoneNumber;
use serde::de::value;
use validator::ValidateEmail;
use validator::ValidateLength;
use validator::ValidateUrl;

use super::email::Email;
use super::long_text_value::LongTextValue;
use super::max_text_length::MAX_TEXT_LENGHT;
use super::phone_value::PhoneValue;
use super::url_value::UrlValue;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SingleLineValue {
    pub(super) value: String,
}

impl ValueType<str> for SingleLineValue {
    fn verify(&self, config: FieldConfig) -> Result<(), ValueError> {
        if let FieldConfig::Text(TextConfig::SingleLine { max_length, .. }) = config {
            let text_lenght = self.value.length().ok_or(ValueError::Unknown)?;
            if text_lenght > max_length.into() {
                Err(ValueError::TextTooBig(text_lenght))
            } else {
                Ok(())
            }
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
                LongText { rich_text } => {
                    let mut value = self.value.clone();
                    if !rich_text {
                        value = value.replace(['*', '_', '#', '`', '[', ']'], "");
                    };
                    Value::LongText(Box::new(LongTextValue { value }))
                };
                Email => {
                    let value = self.value.clone();
                    if value.validate_email() {
                        return Err(ValueError::CantConvertTo("Email".to_string()))
                    };
                    Value::Email(Email { value })
                };
                URL => {
                    let value = self.value.clone();
                    if value.validate_url() {
                        return Err(ValueError::CantConvertTo("Url".to_string()))
                    };
                    Value::URL(UrlValue { value })
                };
                Phone => {
                    let value = self.value.clone();
                    let value = PhoneNumber::from_str(value.as_str())
                        .ok()
                        .ok_or(ValueError::CantConvertTo("Phone number".to_string()))?;
                    Value::Phone(PhoneValue { value: value.format().mode(phonenumber::Mode::E164).to_string() })
                }
            };
            Number {
                Number { default } => {
                    let value = self.value
                        .trim()
                        .split(' ')
                        .find_map(|v| v.parse::<isize>().ok())
                        .ok_or_else(|| ValueError::CantConvertTo("Number".to_string()))?;
                    Value::Number(NumberValue { value })
                };
                Decimal { default, precision} => {
                    let value = self.value
                        .trim()
                        .split(' ')
                        .find_map(|v| v.parse::<f64>().ok())
                        .ok_or_else(|| ValueError::CantConvertTo("Decimal".to_string()))?;
                    Value::Decimal(crate::DecimalValue { value: crate::OrderedFloatIThink(OrderedFloat::from(value))})
                };
                Currency {

                }
            };
        })
    }

    fn value(&self) -> &str {
        &self.value
    }
}

impl SingleLineValue {
    pub fn new(default: Option<String>, value: Option<String>) -> Result<Self, super::ValueError> {
        let raw = value.or(default).ok_or(ValueError::MissingValue)?;
        let single_line = raw.replace(['\n', '\r'], " ");

        let text_lenght = single_line.length().unwrap();
        if text_lenght > MAX_TEXT_LENGHT.into() {
            return Err(super::ValueError::TextTooBig(text_lenght));
        };

        Ok(Self { value: single_line })
    }
}
