use leptos::prelude::*;
use models::{Field, RecordId, Value};

use super::cell::Cell;
use super::field::Field as FieldHeader;

#[component]
pub fn Column(
    field: Field,
    cells: Vec<(RecordId, Value)>,
    on_cell_change: Callback<(RecordId, String, String)>,
    #[prop(optional)] width: Signal<f64>,
    on_resize: Option<Callback<f64>>,
) -> impl IntoView {
    let fname = field.name.clone();
    let fid = field
        .id
        .as_ref()
        .map(|id| id.id_str())
        .unwrap_or_default();
    let fconfig = field.config.clone();
    let on_change = on_cell_change.clone();

    view! {
        <div class="flex flex-col shrink-0">
            <FieldHeader field=field width=width on_resize=on_resize />
            {cells
                .into_iter()
                .map(move |(rid, val)| {
                    view! {
                        <Cell
                            field_config=fconfig.clone()
                            field_name=fid.clone()
                            value=val
                            on_change=on_change.clone()
                            record_id=rid
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}
