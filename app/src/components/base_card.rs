use crate::name_to_hex_color;
use leptos::prelude::*;
use models::Base;

pub async fn BaseCard(base: Base) -> impl IntoView {
    let color = name_to_hex_color(&base.name);
    view! { <div></div> }
}
