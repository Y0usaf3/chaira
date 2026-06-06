use std::str::FromStr;

use crate::CurrencyValue;
use crate::DateValue;
use crate::NumberValue;
use crate::PercentValue;
use crate::RatingValue;
use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use chrono::NaiveTime;
use iso_currency::Currency;
use ordered_float::OrderedFloat;
use phonenumber::PhoneNumber;
use validator::ValidateEmail;
use validator::ValidateLength;
use validator::ValidateUrl;

use super::email::Email;
use super::long_text_value::LongTextValue;
use super::max_text_length::MAX_TEXT_LENGHT;
use super::parse_word;
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
                    let valid_phone = self.value
                        .split_whitespace()
                        .find_map(|v| PhoneNumber::from_str(v).ok())
                        .ok_or(ValueError::CantConvertTo("Phone".to_string()))?;
                    Value::Phone(PhoneValue {
                        value:valid_phone
                            .format().mode(phonenumber::Mode::E164).to_string()
                    })

                }
            };
            Number {
                Number { default } => {
                    Value::Number(NumberValue { value: parse_word(&self.value, "Number")? })
                };
                Decimal { default, precision} => {
                    Value::Decimal(crate::DecimalValue { value: crate::OrderedFloatIThink(OrderedFloat::from(parse_word::<f64>(&self.value, "Decimal")?)) })
                };
                Currency { currency, precision } => {
                    let currency = Currency::from_code(currency).ok_or(ValueError::InvalidCountryCode)?;
                    let cleaned = self.value.replace(&currency.symbol().symbol, " ");
                    Value::Currency(CurrencyValue { value: crate::OrderedFloatIThink(OrderedFloat::from(parse_word::<f64>(&cleaned, "Currency")?)) })
                };
                Percent { precision, show_bar } => {
                    let cleaned = self.value.replace('%', " ");
                    Value::Percent(PercentValue { value: parse_word::<i32>(&cleaned, "Percentage")? })
                };
                Rating { icon_type, max, color } => {
                    let number = parse_word::<u8>(&self.value, "Rating")?;
                    if number as usize > *max {
                        return Err(ValueError::BiggerThanMax);
                    };
                    Value::Rating(RatingValue { value: number })
                }
            };
            Datetime {
                Date { format, include_time } => {
                    let value = if *include_time {
                        format
                            .parse_datetime(&self.value).ok()
                            .ok_or(ValueError::CantConvertTo("Datetime".to_string()))?
                    } else {
                        format.parse_date(&self.value).ok()
                            .ok_or(ValueError::CantConvertTo("Date".to_string()))?.and_time(NaiveTime::MIN)

                    };
                    Value::Date(DateValue {
                        value: Datetime::from(value.and_utc())
                     })
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
