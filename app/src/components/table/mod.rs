use column::Column;
use leptos::prelude::*;
use models::TableId;

mod cell;
mod column;
mod field;

#[component]
pub fn Table(id: TableId) -> impl IntoView {
    view! {}
}
