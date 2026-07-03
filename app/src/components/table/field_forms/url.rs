use crate::components::table::server::create_table_field;
use leptos::{prelude::*, reactive::spawn_local};
use models::{BaseId, FieldConfig, TableId, TextConfig};

#[component]
pub fn URLFieldForm(
    base_id: BaseId,
    table_id: TableId,
    name: ReadSignal<String>,
    set_show: WriteSignal<bool>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (creating, set_creating) = signal(false);

    let handle_create = move |_: leptos::ev::MouseEvent| {
        let base_id = base_id.clone();
        let table_id = table_id.clone();
        let name = name.get_untracked();
        let name = if name.is_empty() { "URL".into() } else { name };

        set_creating.set(true);
        spawn_local(async move {
            if create_table_field(base_id, table_id, name, FieldConfig::Text(TextConfig::URL))
                .await
                .is_ok()
            {
                set_show.set(false);
                on_created.run(());
            } else {
                set_creating.set(false);
            }
        });
    };

    view! {
        <div class="flex flex-col gap-3 p-3 bg-slate-50 border border-slate-200 rounded">
            <p class="text-sm text-slate-500">"No additional configuration needed."</p>
            <button
                type="submit"
                disabled=creating
                class="pixel-corners--wrapper mt-2 p-3 ml-auto bg-black text-white font-bold cursor-pointer w-full text-sm disabled:opacity-50 disabled:cursor-not-allowed"
                on:click=handle_create
            >
                {move || if creating.get() { "Creating..." } else { "Create field" }}
            </button>
        </div>
    }
}
