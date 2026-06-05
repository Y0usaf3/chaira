use std::str::FromStr;

use crate::{Value, ValueError, ValueType, cell::FieldConfig, kinds::TextConfig, prelude::*};
use phonenumber::PhoneNumber;
use validator::ValidateEmail;
use validator::ValidateLength;
use validator::ValidateUrl;

pub const MAX_TEXT_LENGHT: u32 = 999_999; // ~1MB single byte chars

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SingleLineValue {
    value: String,
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
                    let value = PhoneNumber::from_str(value.as_str()).ok().ok_or(ValueError::CantConvertTo("Phone number".to_string()))?;
                    Value::Phone(PhoneValue { value: value.format().mode(phonenumber::Mode::E164).to_string() })
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

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LongTextValue {
    value: String,
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

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Email {
    value: String,
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

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UrlValue {
    value: String,
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
        let value = value
            .split(' ')
            .find(|v| v.validate_url())
            .ok_or(ValueError::InvalidUrl("SORRY".to_string()))?;
        Ok(Self {
            value: value.trim().to_string(),
        })
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PhoneValue {
    value: String,
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
        let value = value
            .split(" ")
            .find(|v| PhoneNumber::from_str(v).is_ok())
            .ok_or(ValueError::InvalidPhoneNumber("SORRY".to_string()))?;

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

// was redesigning the Record/field system since it was poorly made, i made a trait for ValueTypes and a macro to easly write convertion code for each type (also hackatime wouldnt track all da time i spent writing on my note book 3:<)
