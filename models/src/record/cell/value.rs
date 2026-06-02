use crate::prelude::*;
use crate::kinds::{FieldConfig, TextConfig, NumberConfig, DatetimeConfig};
use std::str::FromStr;
use super::text::{SingleLineValue, LongTextValue, Email, UrlValue, PhoneValue};
use super::number::{NumberValue, DecimalValue};
use super::datetime::DateValue;
use super::computed::{FormulaValue, RollUpValue, LookUpValue, LinkValue, AutoNumberValue};
use super::complex::{AttachmentValue, JsonValue};
use super::CellError;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Value {
    SingleLine(SingleLineValue),
    LongText(Box<LongTextValue>),
    Email(Email),
    URL(UrlValue),
    Phone(PhoneValue),
    Number(NumberValue),
    Decimal(DecimalValue),
    Currency(super::number::CurrencyValue),
    Percent(super::number::PercentValue),
    Rating(super::number::RatingValue),
    Date(DateValue),
    Duration(super::datetime::DurationValue),
    Link(LinkValue),
    LookUp(Box<LookUpValue>),
    RollUp(Box<RollUpValue>),
    Formula(Box<FormulaValue>),
    AutoNumber(AutoNumberValue),
    CreatedAt(super::datetime::CreatedAtValue),
    ModifiedTime(super::datetime::ModifiedTimeValue),
    Attachment(Box<AttachmentValue>),
    JSON(Box<JsonValue>),
}

impl Value {
    pub fn convert_to(&self, target_config: &FieldConfig) -> Result<Self, CellError> {
        // 1. To Text (Always safe)
        if let FieldConfig::Text(text_config) = target_config {
            let s = self.to_string();
            return match text_config {
                TextConfig::SingleLine { default, .. } => {
                    SingleLineValue::new(default.clone(), Some(s)).map(Value::SingleLine)
                }
                TextConfig::LongText { rich_text } => {
                    LongTextValue::new(s, *rich_text).map(|v| Value::LongText(Box::new(v)))
                }
                TextConfig::Email => Email::new(s).map(Value::Email),
                TextConfig::URL => UrlValue::new(s).map(Value::URL),
                TextConfig::Phone => PhoneValue::new(s, None).map(Value::Phone),
            };
        }

        // 2. To Number
        if let FieldConfig::Number(num_config) = target_config {
            let s = self.to_string();
            let n_float = s.parse::<f64>().map_err(|_| CellError::MissingValue)?;

            return match num_config {
                NumberConfig::Number { default } => {
                    NumberValue::new(Some(n_float as usize), *default).map(Value::Number)
                }
                NumberConfig::Decimal { default, .. } => {
                    DecimalValue::new(Some(n_float), default.map(|f| f as f64)).map(Value::Decimal)
                }
                _ => Err(CellError::MissingValue),
            };
        }

        // 3. To Datetime
        if let FieldConfig::Datetime(dt_config) = target_config {
            if let DatetimeConfig::Date { .. } = dt_config {
                let s = self.to_string();
                let dt = Datetime::from_str(&s).map_err(|_| CellError::MissingValue)?;
                return Ok(Value::Date(DateValue::new(dt)));
            }
        }

        // 4. Default: Return error if incompatible
        Err(CellError::FieldNotFound(
            "Incompatible types for migration".into(),
        ))
    }

    pub fn to_string(&self) -> String {
        match self {
            Value::SingleLine(v) => v.value().to_string(),
            Value::LongText(v) => v.value().to_string(),
            Value::Email(v) => v.value().to_string(),
            Value::URL(v) => v.value().to_string(),
            Value::Phone(v) => v.value().to_string(),
            Value::Number(v) => v.value().to_string(),
            Value::Decimal(v) => v.value().to_string(),
            Value::Currency(v) => v.value_as_str().to_string(),
            Value::Percent(v) => v.value().to_string(),
            Value::Rating(v) => v.value().to_string(),
            Value::Date(v) => v.value().to_string(),
            Value::Duration(v) => v.value().to_string(),
            Value::CreatedAt(v) => v.value().to_string(),
            Value::ModifiedTime(v) => v.value().to_string(),
            Value::JSON(v) => v.value(),
            _ => "".to_string(), // Fallback for complex types
        }
    }
}
