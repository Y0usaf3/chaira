use crate::{Value, ValueError, ValueType, cell::FieldConfig, kinds::TextConfig, prelude::*};
use validator::ValidateLength;

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
                Err(ValueError::TextTooBig(text_lenght.into()))
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
        match target_config {
            FieldConfig::Text(config) => match config {
                TextConfig::Email => Ok(Value::Email(Email {
                    value: self.value.clone(),
                })),
                _ => Err(ValueError::WrongType(
                    "cant convert to this type".to_string(),
                )),
            },
            _ => Err(ValueError::WrongType(
                "cant convert to this type".to_string(),
            )),
        }
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

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Email {
    value: String,
}

impl Email {
    pub fn new(value: String) -> Result<Self, super::ValueError> {
        if validator::ValidateEmail::validate_email(&value) {
            Ok(Self {
                value: value.trim().to_lowercase(),
            })
        } else {
            Err(super::ValueError::InvalidEmail(value))
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct UrlValue {
    value: String,
}

impl UrlValue {
    pub fn new(value: String) -> Result<Self, super::ValueError> {
        if validator::ValidateUrl::validate_url(&value) {
            Ok(Self {
                value: value.trim().to_string(),
            })
        } else {
            Err(super::ValueError::InvalidUrl(value))
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PhoneValue {
    value: String,
}

impl PhoneValue {
    pub fn new(value: String, default_region: Option<&str>) -> Result<Self, super::ValueError> {
        let region = default_region.and_then(|r| r.parse().ok());

        match phonenumber::parse(region, &value) {
            Ok(phone) => {
                if phonenumber::is_valid(&phone) {
                    let formatted = phone.format().mode(phonenumber::Mode::E164).to_string();
                    Ok(Self { value: formatted })
                } else {
                    Err(super::ValueError::InvalidPhoneNumber(value))
                }
            }
            Err(_) => Err(super::ValueError::UnparseablePhoneNumber(value)),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
