use crate::components::{
    FilteredInput, Popup, Select, SelectOption, table::server::create_table_field,
};
use leptos::{prelude::*, reactive::spawn_local};
use models::{BaseId, FieldConfig, NumberConfig, TableId, TextConfig};

#[derive(Clone, PartialEq, Debug, Default)]
enum FieldType {
    #[default]
    Unselected,
    SingleLine {
        default: Option<String>,
        max_length: u16,
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

    let (selected_key, set_selected_key) = signal(String::new());

    let (sl_max_length, set_sl_max_length) = signal(500);
    let (sl_default, set_sl_default) = signal(String::new());
    let (lt_rich_text, set_lt_rich_text) = signal(false);
    let (num_default, set_num_default) = signal(String::new());

    let active_field_type = Memo::new(move |_| match selected_key.get().as_str() {
        "singleline" => FieldType::SingleLine {
            default: Some(sl_default.get()).filter(|s| !s.is_empty()),
            max_length: sl_max_length.get(),
        },
        "longtext" => FieldType::LongText {
            rich_text: lt_rich_text.get(),
        },
        "email" => FieldType::Email,
        "url" => FieldType::URL,
        "phone" => FieldType::Phone,
        "number" => FieldType::Number {
            default: num_default.get().parse::<isize>().ok(),
        },
        _ => FieldType::Unselected,
    });

    let handle_create_field = move |_: leptos::ev::MouseEvent| {
        let base_id = base_id.clone();
        let table_id = table_id.clone();
        let name = name.get_untracked();

        let config = match active_field_type.get_untracked() {
            FieldType::SingleLine {
                default,
                max_length,
            } => FieldConfig::Text(TextConfig::SingleLine {
                default,
                max_length,
            }),
            FieldType::LongText { rich_text } => {
                FieldConfig::Text(TextConfig::LongText { rich_text })
            }
            FieldType::Email => FieldConfig::Text(TextConfig::Email),
            FieldType::URL => FieldConfig::Text(TextConfig::URL),
            FieldType::Phone => FieldConfig::Text(TextConfig::Phone),
            FieldType::Number { default } => FieldConfig::Number(NumberConfig::Number { default }),
            _ => FieldConfig::Text(TextConfig::SingleLine {
                default: None,
                max_length: 500,
            }),
        };

        spawn_local(async move {
            if create_table_field(base_id, table_id, name, config)
                .await
                .is_ok()
            {
                set_show.set(false);
                set_name.set(String::new());
                set_selected_key.set(String::new());
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
                        getter=selected_key
                        setter=set_selected_key
                    />
                </div>

                {move || match active_field_type.get() {
                    FieldType::SingleLine { .. } => {
                        view! {
                            <div class="flex flex-col gap-3 p-3 bg-slate-50 border-[2px] border-black">
                                <h4 class="text-xs font-bold tex-black uppercase tracking-wider">
                                    "Single Line Configuration"
                                </h4>
                                <div class="flex flex-col gap-1">
                                    <label class="text-xs text-slate-600">"Max Length"</label>
                                    <input
                                        type="number"
                                        class="border p-1 text-sm"
                                        prop:value=sl_max_length
                                        on:input=move |e| {
                                            if let Ok(val) = event_target_value(&e).parse::<u16>() {
                                                set_sl_max_length.set(val);
                                            }
                                        }
                                    />
                                </div>
                                <div class="flex flex-col gap-1">
                                    <label class="text-xs text-slate-600">"Default Value"</label>
                                    <input
                                        type="text"
                                        class="border p-1 text-sm"
                                        prop:value=sl_default
                                        on:input=move |e| set_sl_default.set(event_target_value(&e))
                                    />
                                </div>
                            </div>
                        }
                            .into_any()
                    }
                    FieldType::LongText { .. } => {

                        view! {
                            <div class="flex flex-col gap-3 p-3 bg-slate-50 border-[2px] border-black">
                                <input
                                    type="checkbox"
                                    id="rich_text"
                                    prop:checked=lt_rich_text
                                    on:change=move |e| {
                                        set_lt_rich_text.set(event_target_checked(&e))
                                    }
                                />
                                <label for="rich_text" class="text-sm text-slate-700">
                                    "Enable Rich Text Rendering"
                                </label>
                            </div>
                        }
                            .into_any()
                    }
                    FieldType::Number { .. } => {

                        view! {
                            <div class="flex items-center gap-2 p-3 bg-slate-50 border border-slate-200 rounded">
                                <label class="text-xs text-slate-600">
                                    "Default Numeric Value"
                                </label>
                                <input
                                    type="number"
                                    class="border p-1 text-sm"
                                    prop:value=num_default
                                    on:input=move |e| set_num_default.set(event_target_value(&e))
                                />
                            </div>
                        }
                            .into_any()
                    }
                    _ => view! {}.into_any(),
                }}

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
