use crate::prelude::*;
use validator::ValidateLength;

pub const MAX_TEXT_LENGHT: u32 = 999_999; // ~1MB single byte chars

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct SingleLineValue {
    value: String,
}

impl SingleLineValue {
    pub fn new(default: Option<String>, value: Option<String>) -> Result<Self, super::CellError> {
        let raw = value.or(default).unwrap_or_default();
        let single_line = raw.replace(['\n', '\r'], " ");

        let text_lenght = single_line.length().unwrap();
        if text_lenght > MAX_TEXT_LENGHT.into() {
            return Err(super::CellError::TextTooBig(text_lenght));
        };

        Ok(Self { value: single_line })
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LongTextValue {
    value: String,
}

impl LongTextValue {
    pub fn new(value: String, rich_text: bool) -> Result<Self, super::CellError> {
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
            return Err(super::CellError::TextTooBig(text_lenght));
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
    pub fn new(value: String) -> Result<Self, super::CellError> {
        if validator::ValidateEmail::validate_email(&value) {
            Ok(Self {
                value: value.trim().to_lowercase(),
            })
        } else {
            Err(super::CellError::InvalidEmail(value))
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
    pub fn new(value: String) -> Result<Self, super::CellError> {
        if validator::ValidateUrl::validate_url(&value) {
            Ok(Self {
                value: value.trim().to_string(),
            })
        } else {
            Err(super::CellError::InvalidUrl(value))
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
    pub fn new(value: String, default_region: Option<&str>) -> Result<Self, super::CellError> {
        let region = default_region.and_then(|r| r.parse().ok());

        match phonenumber::parse(region, &value) {
            Ok(phone) => {
                if phonenumber::is_valid(&phone) {
                    let formatted = phone.format().mode(phonenumber::Mode::E164).to_string();
                    Ok(Self { value: formatted })
                } else {
                    Err(super::CellError::InvalidPhoneNumber(value))
                }
            }
            Err(_) => Err(super::CellError::UnparseablePhoneNumber(value)),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}
