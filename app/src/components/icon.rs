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
    Trash,
    Pen,
    NormalUser,
    NotNormalUser,
}

#[component]
pub fn Icon(
    icon_type: IconType,
    #[prop(into, optional)] class: &'static str,
    #[prop(into, optional)] extern_class: Signal<&'static str>,
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
        IconType::Trash => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/trash.svg"
        )),
        IconType::Pen => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/pen.svg"
        )),
        IconType::NormalUser => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/normal_user.svg"
        )),
        IconType::NotNormalUser => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/nor_normal_user.svg"
        )),
        _ => include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../public/svg/unknown.svg"
        )),
    };

    let class_attr = format!(r#"class="{}" viewBox="0 0 32 32""#, class);

    // TODO: use regex for that
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
        .replace(
            r##"fill="#000000""##,
            format!(r##"fill="{fill}""##).as_str(),
        )
        .replace(r##"width="32" height="32""##, &class_attr);

    view! { <span inner_html=svg style="display: inline-block;" class=extern_class></span> }
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
