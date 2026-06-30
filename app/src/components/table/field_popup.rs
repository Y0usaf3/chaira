use crate::components::{
    FilteredInput, Popup, Select, SelectOption, table::server::create_table_field,
};
use leptos::{prelude::*, reactive::spawn_local};
use models::{BaseId, TableId};

enum FieldType {
    SingleLine {
        default: Option<String>,
        max_lenght: u16,
    },
    LongText {
        rich_text: bool,
    },
    Email,
    URL,
    Phone,
    Number {
        default: Option<isize>,
    },
}

#[component]
pub fn CreateFieldPopup(
    base_id: BaseId,
    table_id: TableId,
    show: ReadSignal<bool>,
    set_show: WriteSignal<bool>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (field_type, set_field_type) = signal(String::new());
    let handle_create_field = move |_: leptos::ev::MouseEvent| {
        let base_id = base_id.clone();
        let table_id = table_id.clone();
        let name = name.get_untracked();
        spawn_local(async move {
            if let Ok(_) = create_table_field(
                base_id,
                table_id,
                name,
                models::FieldConfig::Text(models::TextConfig::SingleLine {
                    default: None,
                    max_length: 500,
                }),
            )
            .await
            {
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
            <div class="flex flex-col gap-5">
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
                <div class="flex flex-col gap-1 overflow-visible">
                    <label class="text-sm font-semibold text-slate-700">"Field type"</label>
                    <Select
                        options=vec![
                            SelectOption {
                                value: "singleline".into(),
                                label: "Single Line".into(),
                            },
                            SelectOption {
                                value: "longtext".into(),
                                label: "Long Text".into(),
                            },
                            SelectOption {
                                value: "email".into(),
                                label: "Email".into(),
                            },
                            SelectOption {
                                value: "url".into(),
                                label: "URL".into(),
                            },
                            SelectOption {
                                value: "phone".into(),
                                label: "Phone".into(),
                            },
                            SelectOption {
                                value: "number".into(),
                                label: "Number".into(),
                            },
                        ]
                        getter=field_type
                        setter=set_field_type
                    />
                </div>
                <button
                    type="submit"
                    class="pixel-corners--wrapper mt-2 p-3 ml-auto bg-black text-white font-bold cursor-pointer w-full text-sm"
                    on:click=handle_create_field.clone()
                >
                    "Create field"
                </button>
            </div>
        </Popup>
    }
}
