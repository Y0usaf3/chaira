use super::ValueError;
use super::attachment_value::AttachmentValue;
use super::auto_number_value::AutoNumberValue;
use super::created_at_value::CreatedAtValue;
use super::currency_value::CurrencyValue;
use super::date_value::DateValue;
use super::decimal_value::DecimalValue;
use super::duration_value::DurationValue;
use super::email::Email;
use super::formula_value::FormulaValue;
use super::json_value::JsonValue;
use super::link_value::LinkValue;
use super::long_text_value::LongTextValue;
use super::look_up_value::LookUpValue;
use super::modified_time_value::ModifiedTimeValue;
use super::number_value::NumberValue;
use super::percent_value::PercentValue;
use super::phone_value::PhoneValue;
use super::rating_value::RatingValue;
use super::roll_up_value::RollUpValue;
use super::single_line_value::SingleLineValue;
use super::url_value::UrlValue;
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
    Currency(CurrencyValue),
    Percent(PercentValue),
    Rating(RatingValue),
    Date(DateValue),
    /*     Duration(DurationValue), */
    Link(LinkValue),
    LookUp(Box<LookUpValue>),
    RollUp(Box<RollUpValue>),
    Formula(Box<FormulaValue>),
    AutoNumber(AutoNumberValue),
    CreatedAt(CreatedAtValue),
    ModifiedTime(ModifiedTimeValue),
    Attachment(Box<AttachmentValue>),
    JSON(Box<JsonValue>),
}

impl Value {
    pub fn verify(&self, _config: FieldConfig) -> Result<(), ValueError> {
        Ok(())
    }
}
