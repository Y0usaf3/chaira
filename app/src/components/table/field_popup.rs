use crate::components::{FilteredInput, Popup, table::server::create_table_field};
use leptos::{prelude::*, reactive::spawn_local};
use models::{BaseId, TableId};

#[component]
pub fn CreateFieldPopup(
    base_id: BaseId,
    table_id: TableId,
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let handle_create_field = move |_: leptos::ev::MouseEvent| {
        let base_id = base_id.clone();
        let table_id = table_id.clone();
        let name = name.get_untracked();
        spawn_local(async move {
            if let Ok(_) = create_table_field(base_id, table_id, name, models::FieldConfig::Text(models::TextConfig::SingleLine { default: None, max_length: 500 })).await {
                set_show.set(false);
                set_name.set(String::new());
                on_created.run(());
            }
        });
    };
    view! {
        <Popup show=show.into() set_show=set_show>
            <div class="mb-4 border-b-2 border-slate-200 pb-2 border-dashed">
                <h2 class="text-xl font-bold text-slate-800">"Create Field"</h2>
                <p class="text-sm text-slate-500">"Add a new column to this table."</p>
            </div>
            <div class="flex flex-col gap-4">
                <FilteredInput
                    label="Field Name"
                    placeholder="e.g. Name"
                    value=name
                    set_value=set_name
                    filter=Callback::new(|val: String| {
                        val.chars()
                            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                            .collect()
                    })
                    autofocus=true
                />
            </div>
            <button
                class="pixel-corners--wrapper mt-2 p-3 bg-slate-800 text-white font-bold transition-colors cursor-pointer w-full text-sm"
                on:click=handle_create_field.clone()
            />
        </Popup>
    }
}
