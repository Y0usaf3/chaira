use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconType {
    Email,
    LongText,
    Number,
    Phone,
    SingleLine,
    Url,
}

#[component]
pub fn Icon(icon_type: IconType, class: &'static str) -> impl IntoView {
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
    };
    let svg = raw
        .lines()
        .skip(1)
        .collect::<Vec<_>>()
        .join("\n")
        .replace(r##"fill="#FFFFFF""##, r##"fill="currentColor""##)
        .replace(r##" width="32" height="32""##, r##" viewBox="0 0 32 32""##);
    view! { <div class=class inner_html=svg></div> }
}
