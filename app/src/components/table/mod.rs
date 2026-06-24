use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, NavigateOptions};

#[component]
pub fn Table(base_key: String, table_key: String) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u16); // for now we are going to just refresh everytime the
                                               // user do smt
    let naviguate = use_navigate();
    let naviguate_to_dashboard = move || {
        naviguate("/dashboard", NavigateOptions::default());
    };
    let (base_id, table_id) = match (
        crate::parse_base_id(&base_key),
        crate::parse_table_id(&table_key),
    ) {
        (Ok(bi), Ok(ti)) => (bi, ti),
        _ => {
            naviguate_to_dashboard();
            return view! { <p class="text-red">"INVALID ID(s) !"</p> }.into_any();
        }
    };

    view! {
        <p>{format!("{base_id:?}")}</p>
        <p>{format!("{table_id:?}")}</p>
    }
    .into_any()
}
