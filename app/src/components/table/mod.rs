use std::collections::HashMap;

use column::Column;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use models::{Field, FieldConfig, Record, RecordId as RId, TextConfig, ToSql, Value};

mod cell;
mod column;
mod field;

#[server]
pub async fn get_table_data(
    base_key: String,
    table_key: String,
) -> Result<(Vec<Field>, Vec<Record>), ServerFnError> {
    log!("get_table_data: base={base_key}, table={table_key}");
    use models::surrealdb_types::RecordId;
    let service = crate::get_authenticated_service().await?;
    let base_id = models::BaseId(
        RecordId::parse_simple(&format!("base:{base_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid base id: {e:?}")))?,
    );
    let uid = service.id().clone();
    let table_id = models::TableId(
        RecordId::parse_simple(&format!("table:{table_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid table id: {e:?}")))?,
    );
    let mut ts = service::table::TableService::new(table_id, base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let result = ts
        .get_full_data(None)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get table data: {e:?}")));
    if let Ok((fields, records)) = &result {
        log!(
            "get_table_data: {} fields, {} records",
            fields.len(),
            records.len()
        );
    }
    result
}

#[server]
pub async fn create_table_record(base_key: String, table_key: String) -> Result<Record, ServerFnError> {
    log!("create_table_record: base={base_key}, table={table_key}");
    use models::surrealdb_types::RecordId;
    let service = crate::get_authenticated_service().await?;
    let base_id = models::BaseId(
        RecordId::parse_simple(&format!("base:{base_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid base id: {e:?}")))?,
    );
    let uid = service.id().clone();
    let table_id = models::TableId(
        RecordId::parse_simple(&format!("table:{table_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid table id: {e:?}")))?,
    );
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let insert = models::InsertRecord::new(table_id, HashMap::new());
    let result = ts
        .create_record(insert)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create record: {e:?}")));
    if let Ok(ref rec) = result {
        log!("create_table_record: created record {:?}", rec.id);
    }
    result
}

#[server]
pub async fn create_table_field(
    base_key: String,
    table_key: String,
    name: String,
    config: FieldConfig,
) -> Result<Field, ServerFnError> {
    log!("create_table_field: base={base_key}, table={table_key}, name={name}, config={config:?}");
    use models::surrealdb_types::RecordId;
    let service = crate::get_authenticated_service().await?;
    let base_id = models::BaseId(
        RecordId::parse_simple(&format!("base:{base_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid base id: {e:?}")))?,
    );
    let uid = service.id().clone();
    let table_id = models::TableId(
        RecordId::parse_simple(&format!("table:{table_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid table id: {e:?}")))?,
    );
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let insert = models::InsertField {
        name,
        description: None,
        is_primary: false,
        is_nullable: true,
        is_unique: false,
        order: 0,
        config,
    };
    let result = ts
        .create_field(insert)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create field: {e:?}")));
    if let Ok(ref f) = result {
        log!("create_table_field: created field {:?}", f.id);
    }
    result
}

#[server]
pub async fn update_cell_value(
    base_key: String,
    table_key: String,
    record_id: String,
    field_name: String,
    value_str: String,
) -> Result<Record, ServerFnError> {
    log!("update_cell_value: base={base_key}, table={table_key}, record={record_id}, field={field_name}, value={value_str:?}");
    use models::surrealdb_types::RecordId;
    let service = crate::get_authenticated_service().await?;
    let base_id = models::BaseId(
        RecordId::parse_simple(&format!("base:{base_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid base id: {e:?}")))?,
    );
    let uid = service.id().clone();
    let table_id = models::TableId(
        RecordId::parse_simple(&format!("table:{table_key}"))
            .map_err(|e| ServerFnError::new(format!("Invalid table id: {e:?}")))?,
    );
    let mut ts = service::table::TableService::new(table_id.clone(), base_id, uid)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to create table service: {e:?}")))?;
    let rid = models::RecordId(
        RecordId::parse_simple(&record_id)
            .map_err(|e| ServerFnError::new(format!("Invalid record id: {e:?}")))?,
    );
    let value = if value_str.is_empty() {
        Value::SingleLine(
            models::SingleLineValue::new(None, Some(String::new()))
                .map_err(|e| ServerFnError::new(format!("Value error: {e:?}")))?,
        )
    } else {
        Value::SingleLine(
            models::SingleLineValue::new(None, Some(value_str))
                .map_err(|e| ServerFnError::new(format!("Value error: {e:?}")))?,
        )
    };
    let patch = models::RecordPatch::new(Some(vec![(field_name, value)]));
    let result = ts
        .update_record(rid, patch)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update record: {e:?}")));
    if let Ok(ref rec) = result {
        log!("update_cell_value: updated record {:?}", rec.id);
    }
    result
}

use crate::components::{FilteredInput, PlusIcon, Popup};

#[component]
fn CreateFieldPopup(
    base_key: String,
    table_key: String,
    show: Signal<bool>,
    set_show: WriteSignal<bool>,
    on_created: Callback<()>,
) -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (field_type, set_field_type) = signal(0u8);
    let (max_length, set_max_length) = signal("500".to_string());
    let (rich_text, set_rich_text) = signal(false);

    view! {
        <Popup show=show set_show=set_show>
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
                <div class="flex flex-col gap-2">
                    <label class="text-sm font-semibold text-slate-700">"Field Type"</label>
                    <div class="flex flex-wrap gap-2">
                        <button
                            class="px-3 py-2 text-xs font-medium border border-black rounded transition-colors"
                            class:bg-black=move || field_type.get() == 0
                            class:text-white=move || field_type.get() == 0
                            class:bg-white=move || field_type.get() != 0
                            class:text-slate-700=move || field_type.get() != 0
                            on:click=move |_| set_field_type.set(0)
                        >
                            "Single Line"
                        </button>
                        <button
                            class="px-3 py-2 text-xs font-medium border border-black rounded transition-colors"
                            class:bg-black=move || field_type.get() == 1
                            class:text-white=move || field_type.get() == 1
                            class:bg-white=move || field_type.get() != 1
                            class:text-slate-700=move || field_type.get() != 1
                            on:click=move |_| set_field_type.set(1)
                        >
                            "Long Text"
                        </button>
                        <button
                            class="px-3 py-2 text-xs font-medium border border-black rounded transition-colors"
                            class:bg-black=move || field_type.get() == 2
                            class:text-white=move || field_type.get() == 2
                            class:bg-white=move || field_type.get() != 2
                            class:text-slate-700=move || field_type.get() != 2
                            on:click=move |_| set_field_type.set(2)
                        >
                            "Email"
                        </button>
                        <button
                            class="px-3 py-2 text-xs font-medium border border-black rounded transition-colors"
                            class:bg-black=move || field_type.get() == 3
                            class:text-white=move || field_type.get() == 3
                            class:bg-white=move || field_type.get() != 3
                            class:text-slate-700=move || field_type.get() != 3
                            on:click=move |_| set_field_type.set(3)
                        >
                            "URL"
                        </button>
                    </div>
                </div>
                {move || {
                    if field_type.get() == 0 {
                        view! {
                            <FilteredInput
                                label="Max Length"
                                placeholder="500"
                                value=max_length
                                set_value=set_max_length
                                filter=Callback::new(|val: String| {
                                    val.chars().filter(|c| c.is_ascii_digit()).collect()
                                })
                                autofocus=false
                            />
                        }
                            .into_any()
                    } else if field_type.get() == 1 {
                        view! {
                            <label class="flex items-center gap-2 text-sm font-semibold text-slate-700">
                                <input
                                    type="checkbox"
                                    prop:checked=rich_text
                                    on:input=move |ev| {
                                        set_rich_text.set(event_target_checked(&ev));
                                    }
                                />
                                "Rich Text"
                            </label>
                        }
                            .into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
                <button
                    class="pixel-corners--wrapper mt-2 p-3 bg-slate-800 text-white font-bold transition-colors cursor-pointer w-full text-sm"
                    on:click={
                        let bk = base_key.clone();
                        let tk = table_key.clone();
                        move |_: leptos::ev::MouseEvent| {
                            let nm = name.get_untracked();
                            if nm.is_empty() {
                                return;
                            }
                            let ft = field_type.get_untracked();
                            let ml = max_length.get_untracked().parse::<u16>().unwrap_or(500);
                            let rt = rich_text.get_untracked();
                            spawn_local({
                                let bk = bk.clone();
                                let tk = tk.clone();
                                async move {
                                    let config = match ft {
                                        0 => {
                                            FieldConfig::Text(TextConfig::SingleLine {
                                                default: None,
                                                max_length: ml,
                                            })
                                        }
                                        1 => {
                                            FieldConfig::Text(TextConfig::LongText {
                                                rich_text: rt,
                                            })
                                        }
                                        2 => FieldConfig::Text(TextConfig::Email),
                                        3 => FieldConfig::Text(TextConfig::URL),
                                        _ => {
                                            FieldConfig::Text(TextConfig::SingleLine {
                                                default: None,
                                                max_length: 500,
                                            })
                                        }
                                    };
                                    if let Ok(_) = create_table_field(bk, tk, nm, config).await {
                                        set_show.set(false);
                                        set_name.set(String::new());
                                        on_created.run(());
                                    }
                                }
                            });
                        }
                    }
                >
                    "Create Field"
                </button>
            </div>
        </Popup>
    }
}

#[component]
pub fn Table(base_key: String, table_key: String) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u32);

    let bk_res = base_key.clone();
    let tk_res = table_key.clone();
    let data = Resource::new(
        move || (bk_res.clone(), tk_res.clone(), refresh.get()),
        |(base, table, _)| async move { get_table_data(base, table).await },
    );

    let (col_widths, set_col_widths) = signal::<HashMap<String, f64>>(HashMap::new());

    let bk = base_key.clone();
    let tk = table_key.clone();
    let on_cell_change = Callback::new(
        move |(record_id, field_name, value): (RId, String, String)| {
            let bk = bk.clone();
            let tk = tk.clone();
            let rid_str = record_id.0.key.to_sql();
            spawn_local(async move {
                let _ = update_cell_value(bk, tk, rid_str, field_name, value).await;
            });
        },
    );

    let (show_field_popup, set_show_field_popup) = signal(false);
    let handle_open_field_popup = move |_| set_show_field_popup.set(true);
    let handle_field_created = Callback::new(move |_| set_refresh.update(|v| *v += 1));

    let handle_create_record = {
        let bk = base_key.clone();
        let tk = table_key.clone();
        move |_| {
            let bk = bk.clone();
            let tk = tk.clone();
            spawn_local(async move {
                if let Ok(_) = create_table_record(bk, tk).await {
                    set_refresh.update(|v| *v += 1);
                }
            });
        }
    };

    view! {
        <div class="flex flex-col h-full overflow-hidden bg-white">
            <div class="flex-1 overflow-auto">
                <Suspense fallback=|| {
                    view! {
                        <div class="flex items-center justify-center h-full text-black">
                            "Loading..."
                        </div>
                    }
                }>
                    {move || {
                        let data = data.clone();
                        let col_widths = col_widths.clone();
                        let set_col_widths = set_col_widths.clone();
                        let on_cell_change = on_cell_change.clone();
                        Suspend::new(async move {
                            match data.get() {
                                Some(Ok((fields, records))) => {
                                    let records_empty = records
                                        .iter()
                                        .map(|_| ())
                                        .collect::<Vec<_>>();
                                    let fnames: Vec<String> = fields
                                        .iter()
                                        .map(|f| f.name.clone())
                                        .collect();
                                    let grid_template = Signal::derive(move || {
                                        let cw = col_widths.get();
                                        let sizes: Vec<String> = fnames
                                            .iter()
                                            .map(|n| {
                                                cw.get(n)
                                                    .map(|w| format!("{}px", w))
                                                    .unwrap_or_else(|| "150px".to_string())
                                            })
                                            .collect();
                                        sizes.join(" ")
                                    });

                                    view! {
                                        <div
                                            class="grid border-t-2 border-l-2 border-black"
                                            style=move || {
                                                format!(
                                                    "grid-template-columns: {} 1fr",
                                                    grid_template.get(),
                                                )
                                            }
                                        >
                                            {fields
                                                .into_iter()
                                                .map(move |field| {
                                                    let cells: Vec<_> = records
                                                        .iter()
                                                        .filter_map(|rec| {
                                                            let rid = rec.id.clone()?;
                                                            let val = rec
                                                                .cells
                                                                .get(&field.name)
                                                                .cloned()
                                                                .unwrap_or_else(|| {
                                                                    Value::SingleLine(
                                                                        models::SingleLineValue::new(None, Some(String::new()))
                                                                            .expect("empty string should always be valid"),
                                                                    )
                                                                });
                                                            Some((rid, val))
                                                        })
                                                        .collect();
                                                    let fname_signal = field.name.clone();
                                                    let fname_resize = field.name.clone();
                                                    let w = Signal::derive(move || {
                                                        col_widths
                                                            .get()
                                                            .get(&fname_signal)
                                                            .copied()
                                                            .unwrap_or(150.0)
                                                    });
                                                    let ow = set_col_widths.clone();
                                                    let on_resize = Callback::new(move |new_w: f64| {
                                                        ow.update(|map| {
                                                            map.insert(fname_resize.clone(), new_w);
                                                        });
                                                    });

                                                    view! {
                                                        <Column
                                                            field=field.clone()
                                                            cells=cells
                                                            on_cell_change=on_cell_change.clone()
                                                            width=w
                                                            on_resize=Some(on_resize)
                                                        />
                                                    }
                                                })
                                                .collect_view()}
                                            <div class="flex flex-col shrink-0 w-[32px]">
                                                <button
                                                    class="h-full px-1 border-r-2 border-b-2 border-black bg-white flex items-center justify-center "
                                                    on:click=handle_open_field_popup
                                                >
                                                    <PlusIcon class="w-[16px] h-[16px] pixelated text-black mb-auto mx-auto mt-[8px]" />
                                                </button>
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                }
                                Some(Err(_)) => {
                                    view! {
                                        <p class="text-black p-4">"Failed to load table data"</p>
                                    }
                                        .into_any()
                                }
                                _ => {
                                    view! {                                     <p class="text-black p-4">"Loading..."</p> }
                                        .into_any()
                                }
                            }
                        })
                    }}
                </Suspense>
            </div>
            <CreateFieldPopup
                base_key=base_key.clone()
                table_key=table_key.clone()
                show=show_field_popup.into()
                set_show=set_show_field_popup
                on_created=handle_field_created
            />
            <div class="border-t-2 border-black flex-shrink-0">
                <button
                    class="w-full px-4 py-2 text-sm font-medium text-black flex items-center gap-1 "
                    on:click=handle_create_record
                >
                    <PlusIcon class="w-[14px] h-[14px] pixelated" />
                    "Create Record"
                </button>
            </div>
        </div>
    }
}
