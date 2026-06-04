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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinds::{FieldConfig, TextConfig};

    // ---- SingleLineValue ----

    #[test]
    fn sl_value() {
        let v = SingleLineValue::new(None, Some("hello".into())).unwrap();
        assert_eq!(v.value(), "hello");
    }

    #[test]
    fn sl_verify_ok() {
        let v = SingleLineValue::new(None, Some("hi".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 10,
        });
        assert!(v.verify(cfg).is_ok());
    }

    #[test]
    fn sl_verify_exceeds_max_length() {
        let v = SingleLineValue::new(None, Some("a".repeat(200))).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 10,
        });
        assert_eq!(v.verify(cfg), Err(ValueError::TextTooBig(200)));
    }

    #[test]
    fn sl_verify_wrong_type() {
        let v = SingleLineValue::new(None, Some("hi".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert!(matches!(v.verify(cfg), Err(ValueError::WrongType(_))));
    }

    #[test]
    fn sl_convert_to_long_text_rich() {
        let v = SingleLineValue::new(None, Some("a **b**".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: true });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::LongText(Box::new(LongTextValue {
                value: "a **b**".into()
            }))
        );
    }

    #[test]
    fn sl_convert_to_long_text_plain() {
        let v = SingleLineValue::new(None, Some("a **b**".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: false });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::LongText(Box::new(LongTextValue {
                value: "a b".into()
            }))
        );
    }

    #[test]
    fn sl_convert_to_email_with_valid_email_fails() {
        let v = SingleLineValue::new(None, Some("user@example.com".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::CantConvertTo("Email".to_string()))
        );
    }

    #[test]
    fn sl_convert_to_email_with_invalid_email_succeeds() {
        let v = SingleLineValue::new(None, Some("not-an-email".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::Email(Email {
                value: "not-an-email".into()
            })
        );
    }

    #[test]
    fn sl_convert_to_url_with_valid_url_fails() {
        let v = SingleLineValue::new(None, Some("https://example.com".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::URL);
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::CantConvertTo("Url".to_string()))
        );
    }

    #[test]
    fn sl_convert_to_url_with_invalid_url_succeeds() {
        let v = SingleLineValue::new(None, Some("not-a-url".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::URL);
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::URL(UrlValue {
                value: "not-a-url".into()
            })
        );
    }

    #[test]
    fn sl_convert_to_phone_ok() {
        let v = SingleLineValue::new(None, Some("+14155552671".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Phone);
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::Phone(PhoneValue {
                value: "+14155552671".into()
            })
        );
    }

    #[test]
    fn sl_convert_to_phone_invalid() {
        let v = SingleLineValue::new(None, Some("not-a-phone".into())).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Phone);
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::CantConvertTo("Phone number".to_string()))
        );
    }

    #[test]
    fn sl_convert_to_wrong_outer_group() {
        let v = SingleLineValue::new(None, Some("hi".into())).unwrap();
        let cfg = FieldConfig::Number(crate::kinds::NumberConfig::Number { default: None });
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::WrongType(
                "cant convert to this type".to_string()
            ))
        );
    }

    // ---- LongTextValue ----

    #[test]
    fn lt_value() {
        let v = LongTextValue::new("hello".into(), false).unwrap();
        assert_eq!(v.value(), "hello");
    }

    #[test]
    fn lt_verify_rich_text_ok() {
        let v = LongTextValue::new("**bold**".into(), true).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: true });
        assert!(v.verify(cfg).is_ok());
    }

    #[test]
    fn lt_verify_plain_text_with_rich_fails() {
        let v = LongTextValue::new("**bold**".into(), true).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: false });
        assert_eq!(v.verify(cfg), Err(ValueError::UnallowedRichType));
    }

    #[test]
    fn lt_verify_plain_text_ok() {
        let v = LongTextValue::new("plain text".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: false });
        assert!(v.verify(cfg).is_ok());
    }

    #[test]
    fn lt_verify_wrong_type() {
        let v = LongTextValue::new("hi".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert!(matches!(v.verify(cfg), Err(ValueError::WrongType(_))));
    }

    #[test]
    fn lt_convert_to_single_line_truncates() {
        let v = LongTextValue::new("hello world foo bar".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 5,
        });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::SingleLine(SingleLineValue {
                value: "hello".into()
            })
        );
    }

    #[test]
    fn lt_convert_to_single_line_replaces_newlines() {
        let v = LongTextValue::new("hello\nworld\rfoo".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 100,
        });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::SingleLine(SingleLineValue {
                value: "hello world foo".into()
            })
        );
    }

    #[test]
    fn lt_convert_to_email_finds_valid() {
        let v = LongTextValue::new("some text\nuser@example.com\nmore".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::Email(Email {
                value: "user@example.com".into()
            })
        );
    }

    #[test]
    fn lt_convert_to_email_no_valid() {
        let v = LongTextValue::new("no email here".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::CantConvertTo("Email".to_string()))
        );
    }

    #[test]
    fn lt_convert_to_url_finds_valid() {
        let v = LongTextValue::new("visit https://example.com now".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::URL);
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::URL(UrlValue {
                value: "https://example.com".into()
            })
        );
    }

    #[test]
    fn lt_convert_to_url_no_valid() {
        let v = LongTextValue::new("no url here".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::URL);
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::CantConvertTo("URL".to_string()))
        );
    }

    #[test]
    fn lt_convert_to_phone_finds_valid() {
        let v = LongTextValue::new("call +14155552671 for info".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Phone);
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::Phone(PhoneValue {
                value: "+14155552671".into()
            })
        );
    }

    #[test]
    fn lt_convert_to_phone_no_valid() {
        let v = LongTextValue::new("no phone".into(), false).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Phone);
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::CantConvertTo("Phone".to_string()))
        );
    }

    #[test]
    fn lt_convert_to_wrong_outer_group() {
        let v = LongTextValue::new("hi".into(), false).unwrap();
        let cfg = FieldConfig::Number(crate::kinds::NumberConfig::Number { default: None });
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::WrongType(
                "cant convert to this type".to_string()
            ))
        );
    }

    // ---- Email ----

    #[test]
    fn email_value() {
        let v = Email::new("user@example.com".into()).unwrap();
        assert_eq!(v.value(), "user@example.com");
    }

    #[test]
    fn email_verify_ok() {
        let v = Email::new("user@example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert!(v.verify(cfg).is_ok());
    }

    #[test]
    fn email_verify_invalid() {
        let v = Email {
            value: "not-an-email".into(),
        };
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert_eq!(
            v.verify(cfg),
            Err(ValueError::InvalidEmail("not-an-email".into()))
        );
    }

    #[test]
    fn email_verify_wrong_type() {
        let v = Email::new("user@example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::URL);
        assert!(matches!(v.verify(cfg), Err(ValueError::WrongType(_))));
    }

    #[test]
    fn email_convert_to_single_line() {
        let v = Email::new("user@example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 100,
        });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::SingleLine(SingleLineValue {
                value: "user@example.com".into()
            })
        );
    }

    #[test]
    fn email_convert_to_single_line_truncates() {
        let v = Email::new("user@example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 5,
        });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::SingleLine(SingleLineValue {
                value: "user@".into()
            })
        );
    }

    #[test]
    fn email_convert_to_long_text() {
        let v = Email::new("user@example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: false });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::LongText(Box::new(LongTextValue {
                value: "user@example.com".into()
            }))
        );
    }

    #[test]
    fn email_convert_to_wrong_outer_group() {
        let v = Email::new("user@example.com".into()).unwrap();
        let cfg = FieldConfig::Number(crate::kinds::NumberConfig::Number { default: None });
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::WrongType(
                "cant convert to this type".to_string()
            ))
        );
    }

    // ---- UrlValue ----

    #[test]
    fn url_value() {
        let v = UrlValue::new("https://example.com".into()).unwrap();
        assert_eq!(v.value(), "https://example.com");
    }

    #[test]
    fn url_verify_ok() {
        let v = UrlValue::new("https://example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::URL);
        assert!(v.verify(cfg).is_ok());
    }

    #[test]
    fn url_verify_invalid() {
        let v = UrlValue {
            value: "not-a-url".into(),
        };
        let cfg = FieldConfig::Text(TextConfig::URL);
        assert_eq!(
            v.verify(cfg),
            Err(ValueError::InvalidUrl("not-a-url".into()))
        );
    }

    #[test]
    fn url_verify_wrong_type() {
        let v = UrlValue::new("https://example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert!(matches!(v.verify(cfg), Err(ValueError::WrongType(_))));
    }

    #[test]
    fn url_convert_to_single_line() {
        let v = UrlValue::new("https://example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 100,
        });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::SingleLine(SingleLineValue {
                value: "https://example.com".into()
            })
        );
    }

    #[test]
    fn url_convert_to_single_line_truncates() {
        let v = UrlValue::new("https://example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 10,
        });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::SingleLine(SingleLineValue {
                value: "https://ex".into()
            })
        );
    }

    #[test]
    fn url_convert_to_long_text_rich() {
        let v = UrlValue::new("https://example.com".into()).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: true });
        let res = v.convert_to(&cfg).unwrap();
        let expected = LongTextValue {
            value: "https://example.com".into(),
        };
        assert_eq!(res, Value::LongText(Box::new(expected)));
    }

    #[test]
    fn url_convert_to_wrong_outer_group() {
        let v = UrlValue::new("https://example.com".into()).unwrap();
        let cfg = FieldConfig::Number(crate::kinds::NumberConfig::Number { default: None });
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::WrongType(
                "cant convert to this type".to_string()
            ))
        );
    }

    // ---- PhoneValue ----

    #[test]
    fn phone_value() {
        let v = PhoneValue::new("+14155552671".into(), None).unwrap();
        assert_eq!(v.value(), "+14155552671");
    }

    #[test]
    fn phone_verify_ok() {
        let v = PhoneValue::new("+14155552671".into(), None).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Phone);
        assert!(v.verify(cfg).is_ok());
    }

    #[test]
    fn phone_verify_invalid() {
        let v = PhoneValue {
            value: "not-a-phone".into(),
        };
        let cfg = FieldConfig::Text(TextConfig::Phone);
        assert_eq!(
            v.verify(cfg),
            Err(ValueError::InvalidPhoneNumber("not-a-phone".into()))
        );
    }

    #[test]
    fn phone_verify_wrong_type() {
        let v = PhoneValue::new("+14155552671".into(), None).unwrap();
        let cfg = FieldConfig::Text(TextConfig::Email);
        assert!(matches!(v.verify(cfg), Err(ValueError::WrongType(_))));
    }

    #[test]
    fn phone_convert_to_single_line() {
        let v = PhoneValue::new("+14155552671".into(), None).unwrap();
        let cfg = FieldConfig::Text(TextConfig::SingleLine {
            default: None,
            max_length: 100,
        });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::SingleLine(SingleLineValue {
                value: "+14155552671".into()
            })
        );
    }

    #[test]
    fn phone_convert_to_long_text_creates_phone_value() {
        let v = PhoneValue::new("+14155552671".into(), None).unwrap();
        let cfg = FieldConfig::Text(TextConfig::LongText { rich_text: true });
        let res = v.convert_to(&cfg).unwrap();
        assert_eq!(
            res,
            Value::Phone(PhoneValue {
                value: "+14155552671".into()
            })
        );
    }

    #[test]
    fn phone_convert_to_wrong_outer_group() {
        let v = PhoneValue::new("+14155552671".into(), None).unwrap();
        let cfg = FieldConfig::Number(crate::kinds::NumberConfig::Number { default: None });
        assert_eq!(
            v.convert_to(&cfg),
            Err(ValueError::WrongType(
                "cant convert to this type".to_string()
            ))
        );
    }
}
