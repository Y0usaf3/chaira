use leptos::prelude::*;
use models::{FieldConfig, NumberConfig, TextConfig};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    Email,
    LongText,
    Number,
    Phone,
    SingleLine,
    Url,
    Unknown,
    ChevronDown,
    Check,
}

#[component]
pub fn Icon(
    icon_type: IconType,
    #[prop(into)] class: &'static str,
    fill: &'static str,
) -> impl IntoView {
    let raw = match icon_type {
        IconType::Email => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/email.svg"
        )),
        IconType::LongText => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/long_text.svg"
        )),
        IconType::Number => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/number.svg"
        )),
        IconType::Phone => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/phone.svg"
        )),
        IconType::SingleLine => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/single_line.svg"
        )),
        IconType::Url => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/url.svg"
        )),
        IconType::ChevronDown => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/chevron-down.svg"
        )),
        IconType::Check => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/check.svg"
        )),
        _ => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/unknown.svg"
        )),
    };

    let class_attr = format!(r#"class="{}" viewBox="0 0 16 16""#, class);
    let svg = raw
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .replace(
            r##"fill="#FFFFFF""##,
            format!(r##"fill="{fill}""##).as_str(),
        )
        .replace(
            r##"fill="#1F1F1F""##,
            format!(r##"fill="{fill}""##).as_str(),
        )
        .replace(r##" width="16" height="16""##, &class_attr);

    view! { <span inner_html=svg style="display: contents;"></span> }
}

pub fn field_icon(config: &FieldConfig) -> IconType {
    match config {
        FieldConfig::Text(text_config) => match text_config {
            TextConfig::SingleLine { .. } => IconType::SingleLine,
            TextConfig::LongText { .. } => IconType::LongText,
            TextConfig::Email => IconType::Email,
            TextConfig::URL => IconType::Url,
            TextConfig::Phone => IconType::Phone,
        },
        FieldConfig::Number(_) => IconType::Number,
        _ => IconType::Unknown,
    }
}
