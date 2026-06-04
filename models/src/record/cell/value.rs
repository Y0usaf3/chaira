use super::ValueError;
use super::complex::{AttachmentValue, JsonValue};
use super::computed::{AutoNumberValue, FormulaValue, LinkValue, LookUpValue, RollUpValue};
use super::datetime::DateValue;
use super::number::{DecimalValue, NumberValue};
use super::text::{Email, LongTextValue, PhoneValue, SingleLineValue, UrlValue};
use crate::kinds::FieldConfig;
use crate::prelude::*;

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
    pub fn verify(&self, _config: FieldConfig) -> Result<(), ValueError> {
        Ok(())
    }
}
