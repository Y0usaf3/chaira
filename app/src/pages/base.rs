use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use models::surrealdb_types::RecordId;

#[component]
pub fn BasePage() -> impl IntoView {
    let params = use_params_map();
    let id = match params.read().get("id") {
        Some(id) => id,
        _ => "404".to_string(),
    };
    let base_id = RecordId::parse_simple(format!("base:{id}").as_str());
    view! { <p>{format!("{base_id:?}")}</p> }
}
