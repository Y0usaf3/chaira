use crate::prelude::*;
use thiserror::Error;

#[derive(
    Error, Debug, Clone, PartialEq, Eq, SurrealValue, serde::Serialize, serde::Deserialize,
)]
pub enum ValueError {
    #[error("Invalid email format: {0}")]
    InvalidEmail(String),

    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),

    #[error("Invalid phone number: {0}")]
    InvalidPhoneNumber(String),

    #[error("Unparseable phone number: {0}")]
    UnparseablePhoneNumber(String),

    #[error("Value exceeds maximum rating of {max} ({value})")]
    RatingExceedsMax { value: u8, max: u8 },

    #[error("Required value is missing (both value and default are None)")]
    MissingValue,

    #[error("JSON parsing failed: {0}")]
    InvalidJson(String),

    #[error("Formula evaluation failed: {0}")]
    FormulaEvaluationError(String),

    #[error("Circular reference detected in formula or link")]
    CircularReference,

    #[error("Link error: One-to-One relationship cannot contain multiple IDs")]
    LinkConstraintViolation,

    #[error("Field not found: {0}")]
    FieldNotFound(String),

    #[error("Text too big (lenght: {0})")]
    TextTooBig(u64),

    #[error("Unknown :p")]
    Unknown,

    #[error("Wrong type! got {0}")]
    WrongType(String),

    #[error("Contains rich text when it shouldnt!")]
    UnallowedRichType,

    #[error("Cant convert to {0}")]
    CantConvertTo(String),

    #[error("Impossible to convert to this type")]
    Impossible,

    #[error("Invalid country code")]
    InvalidCountryCode,
}
