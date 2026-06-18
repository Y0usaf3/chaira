use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

#[component]
pub fn BasePage() -> impl IntoView {
    let params = use_params_map();
    let id = params.read().get("id");
    view! { <p>{format!("{id:?}")}</p> }
}
