/// Cell module containing all cell value types and related structures
/// 
/// This module is organized into separate concerns:
/// - `error`: CellError enum
/// - `cell_value`: CellValue wrapper struct
/// - `value`: Main Value enum and conversion logic
/// - `text`: Text-based field types (SingleLine, LongText, Email, URL, Phone)
/// - `number`: Numeric field types (Number, Decimal, Percent, Currency, Rating)
/// - `datetime`: Date and time field types (Date, Duration, CreatedAt, ModifiedTime)
/// - `computed`: Computed and relation field types (Formula, RollUp, LookUp, Link, AutoNumber)
/// - `complex`: Complex field types (Attachment, JSON)

pub mod error;
pub mod cell_value;
pub mod value;
pub mod text;
pub mod number;
pub mod datetime;
pub mod computed;
pub mod complex;

// Re-export for convenience
pub use error::CellError;
pub use cell_value::CellValue;
pub use value::Value;

// Text types
pub use text::{SingleLineValue, LongTextValue, Email, UrlValue, PhoneValue, MAX_TEXT_LENGHT};

// Number types
pub use number::{NumberValue, DecimalValue, PercentValue, CurrencyValue, RatingValue, OrderedFloatIThink};

// DateTime types
pub use datetime::{DateValue, DurationValue, CreatedAtValue, ModifiedTimeValue};

// Computed types
pub use computed::{FormulaValue, RollUpValue, LookUpValue, LinkValue, AutoNumberValue};

// Complex types
pub use complex::{AttachmentItem, AttachmentValue, JsonValue, Meme};
