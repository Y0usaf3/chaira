use leptos::prelude::*;
use models::{Field, RecordId, Value};

struct Cells(Vec<(RecordId, Value)>);

#[component]
pub fn Column(field: Field, cells: Cells) -> impl IntoView {
    view! { <div class="flex flex-col"></div> }
}
