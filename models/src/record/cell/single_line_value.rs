use std::str::FromStr;

use crate::CurrencyValue;
use crate::DateValue;
use crate::NumberValue;
use crate::PercentValue;
use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use chrono::DateTime;
use chrono::Local;
use chrono::NaiveTime;
use chrono::TimeZone;
use chrono::Utc;
use iso_currency::Currency;
use ordered_float::OrderedFloat;
use phonenumber::PhoneNumber;
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
                Currency { currency, precision } => {
                    let currency = Currency::from_code(currency).ok_or(ValueError::InvalidCountryCode)?;
                    let value = self.value.replace(&currency.symbol().symbol, " ")
                        .trim()
                        .split(' ')
                        .find_map(|v| v.parse::<f64>().ok())
                        .ok_or_else(|| ValueError::CantConvertTo("Currency".to_string()))?;
                    Value::Currency(CurrencyValue { value: crate::OrderedFloatIThink(OrderedFloat::from(value)) })
                };
                Percent { precision, show_bar } => {
                    let value = self.value
                        .replace('%', " ")
                        .split(' ')
                        .find_map(|v| v.parse::<i32>().ok())
                        .ok_or_else(|| ValueError::CantConvertTo("Percentage".to_string()))?;
                    Value::Percent(PercentValue { value })
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
