use leptos::prelude::*;
use models::{RecordId, Value};

#[component]
pub fn Column(
    children: Children,
    on_cell_change: Callback<(RecordId, String, String)>,
) -> impl IntoView {
    view! {
        move ||
        {children()}
    }
}
