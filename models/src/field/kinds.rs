use chrono::{NaiveDate, NaiveDateTime, ParseError};
use iso_currency::Currency;

use crate::prelude::*;

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FieldConfig {
    Text(TextConfig),
    Number(NumberConfig),
    Select(SelectConfig),
    Datetime(DatetimeConfig),
    Relation(RelationConfig),
    User(UserConfig),
    Computed(ComputedTypes),
    Custom(CustomConfig),
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextConfig {
    SingleLine {
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<String>,
        max_length: u16,
    },
    LongText {
        rich_text: bool,
    },
    Email,
    URL,
    Phone,
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NumberConfig {
    Number {
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<usize>,
    },
    Decimal {
        #[serde(skip_serializing_if = "Option::is_none")]
        default: Option<f32>,
        precision: u8,
    },
    Currency {
        currency: String, // code e.g 'USD' 'XXX'
        precision: usize, /* 0 - 8*/
    },
    Percent {
        precision: usize, /* 0 - 8 */
        show_bar: bool,
    },
    Rating {
        max: usize, /* 0- 10 */
        icon_type: RatingIcon,
        color: [u8; 3], // thats enough for colors
    },
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RatingIcon {
    Star,
    Heart,
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectOption {
    pub id: u8,
    pub label: String,
    pub color: [u8; 6],
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SelectConfig {
    Single { options: Vec<SelectOption> },
    Multi { options: Vec<SelectOption> },
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DateFormat {
    ISO,
    US,
    EU,
}

impl DateFormat {
    pub fn parse_date(&self, date_str: &str) -> Result<NaiveDate, ParseError> {
        match self {
            DateFormat::ISO => NaiveDate::parse_from_str(date_str, "%Y-%m-%d"),

            DateFormat::US => NaiveDate::parse_from_str(date_str, "%m/%d/%Y")
                .or_else(|_| NaiveDate::parse_from_str(date_str, "%m-%d-%Y")),

            DateFormat::EU => NaiveDate::parse_from_str(date_str, "%d/%m/%Y")
                .or_else(|_| NaiveDate::parse_from_str(date_str, "%d-%m-%Y")),
        }
    }
    pub fn parse_datetime(&self, datetime_str: &str) -> Result<NaiveDateTime, ParseError> {
        match self {
            DateFormat::ISO => NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%dT%H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")),

            DateFormat::US => NaiveDateTime::parse_from_str(datetime_str, "%m/%d/%Y %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%m-%d-%Y %H:%M:%S")),

            DateFormat::EU => NaiveDateTime::parse_from_str(datetime_str, "%d/%m/%Y %H:%M:%S")
                .or_else(|_| NaiveDateTime::parse_from_str(datetime_str, "%d-%m-%Y %H:%M:%S")),
        }
    }
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TimeUnits {
    Sec,
    Min,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DatetimeConfig {
    Date {
        format: DateFormat,
        include_time: bool,
    },
    // Duration {
    //     unit: TimeUnits,
    //     format: DateFormat,
    // },
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkType {
    OneToOne,
    OneToMany,
    ManyToMany,
}

#[derive(SurrealValue, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggregateFunction {
    Count,
    Max,
    Min,
    Avg,
    Sum,
    CountDistinct,
    SumDistinct,
    AvgDistinct,
}

#[derive(Debug, Clone, PartialEq, SurrealValue, Serialize, Deserialize)]
pub enum RelationConfig {
    Link {
        target_table_id: TableId,
        r#type: LinkType,
        inverse_field_id: FieldId,
    },
    LookUp {
        link_field_id: FieldId,
        target_field_id: FieldId,
    },
    RollUp {
        link_field_id: FieldId,
        target_field_id: FieldId,
        functions: AggregateFunction,
    },
}

#[derive(Debug, Clone, PartialEq, SurrealValue, Serialize, Deserialize)]
pub enum UserConfig {
    User { is_multi: bool, notify: bool },
}

#[derive(Debug, Clone, PartialEq, SurrealValue, Serialize, Deserialize)]
pub enum ComputedTypes {
    Formula { expression: String },
    CreatedAt { format: DateFormat },
    ModifiedTime { format: DateFormat },
    AutoNumber { prefix: Prefix, start_at: usize },
}

#[derive(Debug, Clone, PartialEq, SurrealValue, Eq, Hash, Serialize, Deserialize)]
pub enum Prefix {
    Dot,
    Star,
}

#[derive(Debug, Clone, PartialEq, SurrealValue, Serialize, Deserialize)]
pub enum CustomConfig {
    Attachment,
    JSON,
}
