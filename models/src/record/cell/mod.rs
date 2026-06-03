#[macro_export]
macro_rules! try_convert {
    // Entry: `target_config { group1 group2 ... }`
    ($cfg:ident { $($groups:tt)* }) => {
        $crate::try_convert!(@start $cfg ; $($groups)*)
    };

    // ---- outer group dispatch ----

    // Pop one outer group, process it, then continue with rest
    (@start $cfg:ident ; $outer:ident { $($inner:tt)* } $($rest:tt)*) => {
        $crate::try_convert!(@grp $cfg ; $outer ; $($inner)* ; $($rest)*)
    };

    // No more groups – final fallback
    (@start $cfg:ident ;) => {
        Err($crate::ValueError::WrongType(
            "cant convert to this type".to_string(),
        ))
    };

    // ---- inner arm processing ----

    // inner arm with destructuring; arm ends with `;`
    (@grp $cfg:ident ; $outer:ident ; $sub:ident { $($field:ident),* } => $result:expr ; $($rest:tt)*) => {
        #[allow(unused_variables)]
        if let $crate::kinds::FieldConfig::$outer(
            $crate::try_convert!(@inner_path $outer ; $sub { $($field),* }),
        ) = $cfg {
            Ok($result)
        } else {
            $crate::try_convert!(@grp $cfg ; $outer ; $($rest)*)
        }
    };

    // inner unit arm; arm ends with `;`
    (@grp $cfg:ident ; $outer:ident ; $sub:ident => $result:expr ; $($rest:tt)*) => {
        if let $crate::kinds::FieldConfig::$outer($crate::try_convert!(@inner_path $outer ; $sub)) = $cfg {
            Ok($result)
        } else {
            $crate::try_convert!(@grp $cfg ; $outer ; $($rest)*)
        }
    };

    // no more inner arms – go to next outer group
    (@grp $cfg:ident ; $outer:ident ; ; $($rest:tt)*) => {
        $crate::try_convert!(@start $cfg ; $($rest)*)
    };

    // ---- inner path helpers (each expands to a complete pattern path) ----
    (@inner_path Text ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::TextConfig::$sub { $($field),* }
    };
    (@inner_path Text ; $sub:ident) => {
        $crate::kinds::TextConfig::$sub
    };
    (@inner_path Number ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::NumberConfig::$sub { $($field),* }
    };
    (@inner_path Number ; $sub:ident) => {
        $crate::kinds::NumberConfig::$sub
    };
    (@inner_path Select ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::SelectConfig::$sub { $($field),* }
    };
    (@inner_path Select ; $sub:ident) => {
        $crate::kinds::SelectConfig::$sub
    };
    (@inner_path Datetime ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::DatetimeConfig::$sub { $($field),* }
    };
    (@inner_path Datetime ; $sub:ident) => {
        $crate::kinds::DatetimeConfig::$sub
    };
    (@inner_path Relation ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::RelationConfig::$sub { $($field),* }
    };
    (@inner_path Relation ; $sub:ident) => {
        $crate::kinds::RelationConfig::$sub
    };
    (@inner_path User ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::UserConfig::$sub { $($field),* }
    };
    (@inner_path User ; $sub:ident) => {
        $crate::kinds::UserConfig::$sub
    };
    (@inner_path Computed ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::ComputedTypes::$sub { $($field),* }
    };
    (@inner_path Computed ; $sub:ident) => {
        $crate::kinds::ComputedTypes::$sub
    };
    (@inner_path Custom ; $sub:ident { $($field:ident),* }) => {
        $crate::kinds::CustomConfig::$sub { $($field),* }
    };
    (@inner_path Custom ; $sub:ident) => {
        $crate::kinds::CustomConfig::$sub
    };
}

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
