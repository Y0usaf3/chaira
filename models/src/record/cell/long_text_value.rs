use std::str::FromStr;

use crate::Value;
use crate::ValueError;
use crate::ValueType;
use crate::cell::FieldConfig;
use crate::kinds::TextConfig;
use crate::prelude::*;
use crate::*;
use chrono::NaiveTime;
use iso_currency::Currency;
use ordered_float::OrderedFloat;
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
#[surreal(crate = "::surrealdb_types")]
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
                        .split_whitespace()
                        .find(|v| v.validate_email())
                        .ok_or(ValueError::CantConvertTo("Email".to_string()))?
                        .to_string();
                    Value::Email(Email { value: valid_email })
                };
                URL => {
                    let valid_url = self.value
                        .split_whitespace()
                        .find(|v| v.validate_url())
                        .ok_or(ValueError::CantConvertTo("URL".to_string()))?
                        .to_string();
                    Value::URL(UrlValue { value: valid_url })
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
