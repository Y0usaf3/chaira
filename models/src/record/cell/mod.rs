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

use std::str::FromStr;

pub(crate) fn parse_word<T: FromStr>(s: &str, label: &str) -> Result<T, ValueError> {
    s.trim()
        .split(' ')
        .find_map(|v| v.parse().ok())
        .ok_or_else(|| ValueError::CantConvertTo(label.into()))
}

pub mod attachment_item;
pub mod attachment_value;
pub mod auto_number_value;
pub mod created_at_value;
pub mod currency_value;
pub mod date_value;
pub mod decimal_value;
pub mod duration_value;
pub mod email;
pub mod error;
pub mod formula_value;
pub mod json_value;
pub mod link_value;
pub mod long_text_value;
pub mod look_up_value;
pub mod max_text_length;
pub mod meme;
pub mod modified_time_value;
pub mod number_value;
pub mod ordered_float_i_think;
pub mod percent_value;
pub mod phone_value;
pub mod rating_value;
pub mod roll_up_value;
pub mod single_line_value;
pub mod url_value;
pub mod value;
pub use attachment_item::AttachmentItem;
pub use attachment_value::AttachmentValue;
pub use auto_number_value::AutoNumberValue;
pub use created_at_value::CreatedAtValue;
pub use currency_value::CurrencyValue;
pub use date_value::DateValue;
pub use decimal_value::DecimalValue;
pub use duration_value::DurationValue;
pub use email::Email;
pub use error::ValueError;
pub use formula_value::FormulaValue;
pub use json_value::JsonValue;
pub use link_value::LinkValue;
pub use long_text_value::LongTextValue;
pub use look_up_value::LookUpValue;
pub use max_text_length::MAX_TEXT_LENGHT;
pub use meme::Meme;
pub use modified_time_value::ModifiedTimeValue;
pub use number_value::NumberValue;
pub use ordered_float_i_think::OrderedFloatIThink;
pub use percent_value::PercentValue;
pub use phone_value::PhoneValue;
pub use rating_value::RatingValue;
pub use roll_up_value::RollUpValue;
pub use single_line_value::SingleLineValue;
pub use url_value::UrlValue;
pub use value::Value;

use crate::kinds::FieldConfig;

pub trait ValueType<T: ?Sized> {
    fn verify(&self, config: FieldConfig) -> Result<(), ValueError>;
    fn convert_to(&self, target_config: &FieldConfig) -> Result<Value, ValueError>
    where
        Self: Sized;
    fn value(&self) -> &T;
}
