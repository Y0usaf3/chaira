use leptos::prelude::*;
use models::{RecordId, Value};

#[component]
pub fn Column(
    children: Children,
    on_cell_change: Callback<(RecordId, String, String)>,
    #[prop(optional)] width: Signal<f64>,
    on_resize: Option<Callback<f64>>,
) -> impl IntoView {
    view! { <div class="flex flex-col shrink-0">move || {children()}</div> }
}
