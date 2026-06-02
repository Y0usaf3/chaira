pub mod cell_value;
pub mod complex;
pub mod computed;
pub mod datetime;
pub mod error;
pub mod number;
pub mod text;
pub mod value;
pub use cell_value::CellValue;
pub use complex::{AttachmentItem, AttachmentValue, JsonValue, Meme};
pub use computed::{AutoNumberValue, FormulaValue, LinkValue, LookUpValue, RollUpValue};
pub use datetime::{CreatedAtValue, DateValue, DurationValue, ModifiedTimeValue};
pub use error::ValueError;
pub use number::{
    CurrencyValue, DecimalValue, NumberValue, OrderedFloatIThink, PercentValue, RatingValue,
};
pub use text::{Email, LongTextValue, MAX_TEXT_LENGHT, PhoneValue, SingleLineValue, UrlValue};
pub use value::Value;

use crate::kinds::FieldConfig;

pub trait ValueType<T: ?Sized> {
    fn verify(&self, config: FieldConfig) -> Result<(), ValueError>;
    fn convert_to(&self, target_config: &FieldConfig) -> Result<Value, ValueError>
    where
        Self: Sized;
    fn value(&self) -> &T;
}
