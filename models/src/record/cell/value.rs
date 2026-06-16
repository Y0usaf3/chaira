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
use crate::{ValueType, prelude::*};

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
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
    pub fn verify(&self, config: FieldConfig) -> Result<(), ValueError> {
        match self {
            Self::SingleLine(v) => v.verify(config),
            Self::LongText(v) => v.verify(config),
            Self::Email(v) => v.verify(config),
            Self::URL(v) => v.verify(config),
            Self::Phone(v) => v.verify(config),
            Self::Number(v) => v.verify(config),
            Self::Decimal(v) => v.verify(config),
            Self::Currency(v) => v.verify(config),
            Self::Percent(v) => v.verify(config),
            Self::Rating(v) => v.verify(config),
            Self::Date(v) => v.verify(config),
            // Self::Duration(v) => v.verify(config), // we still gonna support those types on day
            // :p
            // Self::Link(v) => v.verify(config),
            // Self::LookUp(v) => v.verify(config),
            // Self::RollUp(v) => v.verify(config),
            // Self::Formula(v) => v.verify(config),
            // Self::AutoNumber(v) => v.verify(config),
            // Self::CreatedAt(v) => v.verify(config),
            // Self::ModifiedTime(v) => v.verify(config),
            // Self::Attachment(v) => v.verify(config),
            // Self::JSON(v) => v.verify(config),
            _ => Err(ValueError::Unknown),
        }
    }

    pub fn convert_to(&self, target_config: &FieldConfig) -> Result<Value, ValueError> {
        match self {
            Self::SingleLine(v) => v.convert_to(target_config),
            Self::LongText(v) => v.convert_to(target_config),
            Self::Email(v) => v.convert_to(target_config),
            Self::URL(v) => v.convert_to(target_config),
            Self::Phone(v) => v.convert_to(target_config),
            Self::Number(v) => v.convert_to(target_config),
            Self::Decimal(v) => v.convert_to(target_config),
            Self::Currency(v) => v.convert_to(target_config),
            Self::Percent(v) => v.convert_to(target_config),
            Self::Rating(v) => v.convert_to(target_config),
            Self::Date(v) => v.convert_to(target_config),
            // Self::Duration(v) => v.convert_to(target_config),
            // Self::Link(v) => v.convert_to(target_config),
            // Self::LookUp(v) => v.convert_to(target_config),
            // Self::RollUp(v) => v.convert_to(target_config),
            // Self::Formula(v) => v.convert_to(target_config),
            // Self::AutoNumber(v) => v.convert_to(target_config),
            // Self::CreatedAt(v) => v.convert_to(target_config),
            // Self::ModifiedTime(v) => v.convert_to(target_config),
            // Self::Attachment(v) => v.convert_to(target_config),
            // Self::JSON(v) => v.convert_to(target_config),
            _ => Err(ValueError::Unknown),
        }
    }
}

// alr so abt the field version thingy idk, what we can do is that we set a version of the type,
// live version = 1, and it always uses the current type, but when its version = 2 now, and there is
// a version 1 cell, we apply a mogration code to automaticly migrate the cell from version 1 to
// version 2, so it always handles the latest type, since all the type code is well, on the server
// side :p
// oki so lemem read what i was doing lately abt the field thing
