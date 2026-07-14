use std::collections::HashMap;

use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos_router::{NavigateOptions, hooks::use_navigate};
/* use leptos::logging::log; */

mod cell;
mod column;
mod field;
mod field_popup;
mod server;

use server::get_table_data;

use self::cell::Cell;
use self::column::Column;
use crate::components::table::{
    field::Field,
    field_popup::CreateFieldPopup,
    server::{create_table_record, delete_field, rename_field, update_cell_value},
};
use crate::components::{FilteredInput, PlusIcon, Popup};
use models::{FieldConfig, FieldId, SingleLineValue, Value};

#[component]
pub fn Table(base_key: String, table_key: String) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u16);
    let (show_field_popup, set_show_field_popup) = signal(false);
    let handle_open_field_popup = move |_| set_show_field_popup.set(true);
    let handle_field_created = Callback::new(move |_| set_refresh.update(|v| *v += 1));

    let navigate = use_navigate();
    let navigate_to_dashboard = move || {
        navigate("/dashboard", NavigateOptions::default());
    };

    let parsed_ids = match (
        crate::parse_base_id(&base_key),
        crate::parse_table_id(&table_key),
    ) {
        (Ok(bi), Ok(ti)) => Some((bi, ti)),
        _ => None,
    };

    let data = {
        let parsed = parsed_ids.clone();
        Resource::new(
            move || (parsed.clone(), refresh.get()),
            |(parsed_opt, _)| async move {
                if let Some((base, table)) = parsed_opt {
                    get_table_data(base, table).await
                } else {
                    Ok((vec![], vec![]))
                }
            },
        )
    };

    let (base_id, table_id) = if let Some(ids) = parsed_ids.clone() {
        ids
    } else {
        navigate_to_dashboard();
        return ().into_any();
    };

    let (bi, ti) = (base_id.clone(), table_id.clone());
    let (bii, tii) = (base_id.clone(), table_id.clone());

    let on_cell_change = Callback::new(
        move |(record_id, field_id, value): (models::RecordId, models::FieldId, models::Value)| {
            let bk = bi.clone(); // dont ask me i hate this
            let tk = ti.clone();
            spawn_local(async move {
                let _ = update_cell_value(bk, tk, record_id, field_id, value).await;
            });
        },
    );

    let delete_field = Callback::new(move |field_id: models::FieldId| {
        let bk = bii.clone();
        let tk = tii.clone();
        spawn_local(async move {
            if delete_field(bk, tk, field_id).await.is_ok() {
                set_refresh.update(|v| *v += 1);
            };
        })
    });

    let (show_rename_popup, set_show_rename_popup) = signal(false);
    let (rename_field_id, set_rename_field_id) = signal::<Option<models::FieldId>>(None);
    let (rename_name, set_rename_name) = signal(String::new());

    let (biii, tiii) = (base_id.clone(), table_id.clone());

    let on_rename = Callback::new(move |(field_id, field_name): (models::FieldId, String)| {
        set_rename_field_id.set(Some(field_id));
        set_rename_name.set(field_name);
        set_show_rename_popup.set(true);
    });

    let on_rename_submit = Callback::new(move |_: ()| {
        let field_id = rename_field_id.get_untracked();
        let new_name = rename_name.get_untracked();
        if let (Some(fid), name) = (field_id, new_name) {
            if !name.is_empty() {
                let bk = biii.clone();
                let tk = tiii.clone();
                spawn_local(async move {
                    if rename_field(bk, tk, fid, name).await.is_ok() {
                        set_refresh.update(|v| *v += 1);
                    };
                });
                set_show_rename_popup.set(false);
                set_rename_field_id.set(None);
                set_rename_name.set(String::new());
            }
        }
    });

    let (col_widths, set_col_widths) = signal::<HashMap<String, f64>>(HashMap::new());

    view! {
        {move || match parsed_ids.clone() {
            None => {
                navigate_to_dashboard();
                view! { <p class="text-red">"INVALID ID(s) !"</p> }.into_any()
            }
            Some((base_id, table_id)) => {
                let bid = base_id.clone();
                let tid = table_id.clone();
                view! {
                    <Transition fallback=move || {
                        view! { <p>"Loading..."</p> }
                    }>
                        <div class="flex flex-col w-fit h-full overflow-visible">

                            {move || {
                                let data = data.get();
                                let (fields, records) = match data {
                                    Some(Ok(v)) => v,
                                    _ => (vec![], vec![]),
                                };
                                let field_keys: Vec<String> = fields
                                    .iter()
                                    .filter_map(|f| f.id.as_ref().map(|id| id.id_str()))
                                    .collect();
                                let headers: Vec<_> = fields
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(i, field)| {
                                        let key = field_keys.get(i)?.clone();
                                        let k_for_w = key.clone();
                                        let w = Signal::derive({
                                            move || {
                                                col_widths.get().get(&k_for_w).copied().unwrap_or(200.0)
                                            }
                                        });
                                        let or = {
                                            let k = key.clone();
                                            Callback::new(move |new_w: f64| {
                                                set_col_widths
                                                    .update(|map| {
                                                        map.insert(k.clone(), new_w);
                                                    });
                                            })
                                        };
                                        Some(

                                            view! {
                                                <Field
                                                    field=field.clone()
                                                    width=w
                                                    on_resize=Some(or)
                                                    delete_field_thingy=delete_field
                                                    on_rename=on_rename
                                                />
                                            },
                                        )
                                    })
                                    .collect();
                                let field_ids: Vec<Option<FieldId>> = fields
                                    .iter()
                                    .map(|f| f.id.clone())
                                    .collect();
                                let field_configs: Vec<FieldConfig> = fields
                                    .iter()
                                    .map(|f| f.config.clone())
                                    .collect();
                                let columns: Vec<_> = field_keys
                                    .iter()
                                    .enumerate()
                                    .map({
                                        let records = records.clone();
                                        let field_ids = field_ids.clone();
                                        let field_configs = field_configs.clone();
                                        move |(i, key)| {
                                            let k_for_w = key.clone();
                                            let w = Signal::derive({
                                                move || {
                                                    col_widths.get().get(&k_for_w).copied().unwrap_or(200.0)
                                                }
                                            });
                                            let cells: Vec<_> = records
                                                .iter()
                                                .filter_map({
                                                    let field_ids = field_ids.clone();
                                                    let field_configs = field_configs.clone();
                                                    move |record| {
                                                        let rid = record.id.clone()?;
                                                        let fid = field_ids[i].clone()?;
                                                        let val = record
                                                            .cells
                                                            .get(&field_ids[i].as_ref()?.id_str())
                                                            .cloned()
                                                            .unwrap_or_else(|| {
                                                                Value::SingleLine(
                                                                    SingleLineValue::new(None, Some(String::new()))
                                                                        .expect("empty string is always valid"),
                                                                )
                                                            });
                                                        let cfg = field_configs[i].clone();
                                                        Some(

                                                            view! {
                                                                <Cell
                                                                    field_config=cfg
                                                                    field_name=fid
                                                                    value=val
                                                                    on_change=on_cell_change
                                                                    record_id=rid
                                                                />
                                                            },
                                                        )
                                                    }
                                                })
                                                .collect();

                                            view! { <Column width=w>{cells.into_view()}</Column> }
                                        }
                                    })
                                    .collect();
                                view! {
                                    <div class="flex">
                                        <div class="flex flex-col">
                                            <div
                                                class="flex flex-row items-center h-[38px] w-fit border-black border-b-[2px] divide-x-[2px] divide-black sticky top-0 bg-white"
                                                style:z-index="11"
                                            >
                                                {headers.into_view()}
                                            </div>
                                            <div class="flex flex-row flex-1 w-fit divide-x-[2px] divide-black overflow-auto">
                                                {columns.into_view()}
                                            </div>
                                        </div>
                                        <button
                                            on:click=handle_open_field_popup
                                            class="sticky right-0 bg-white flex items-center justify-center h-full px-[7px] pt-[7px] border-black border-r-[2px] border-l-[2px] transition-colors shrink-0 z-14"
                                        >
                                            <PlusIcon class="w-[16px] h-[16px] pixelated text-black mb-auto mx-auto" />
                                        </button>
                                    </div>
                                }
                                    .into_any()
                            }}
                            <button
                                class="sticky bottom-0 pl-[7px] py-[7px] border-black border-b-[2px] border-r-[2px] border-t-[2px] bg-white"
                                style="position: sticky; bottom: 0; z-index: 10;"
                                on:click=move |_| {
                                    let bk = base_id.clone();
                                    let tk = table_id.clone();
                                    spawn_local(async move {
                                        if create_table_record(bk, tk).await.is_ok() {
                                            set_refresh.update(|v| *v += 1);
                                        }
                                    });
                                }
                            >

                                <PlusIcon class="w-[16px] h-[16px] pixelated fill-black my-auto mr-auto" />
                            </button>
                            <CreateFieldPopup
                                base_id=bid
                                table_id=tid
                                show=show_field_popup
                                set_show=set_show_field_popup
                                on_created=handle_field_created
                            /> <Show when=move || show_rename_popup.get()>
                                <Popup show=show_rename_popup.into() set_show=set_show_rename_popup>
                                    <div class="mb-4 border-b-2 border-slate-200 pb-2 border-dashed">
                                        <h2 class="text-xl font-bold text-slate-800">
                                            "Rename Field"
                                        </h2>
                                        <p class="text-sm text-slate-500">
                                            "Change the name of this column."
                                        </p>
                                    </div>
                                    <div class="flex flex-col gap-5">
                                        <FilteredInput
                                            label="Field Name"
                                            placeholder="e.g. Name"
                                            value=rename_name
                                            set_value=set_rename_name
                                            filter=Callback::new(|val: String| {
                                                val.chars()
                                                    .filter(|c| {
                                                        c.is_ascii_alphanumeric() || *c == '_' || *c == '-'
                                                    })
                                                    .collect()
                                            })
                                            autofocus=true
                                        />
                                        <button
                                            type="submit"
                                            class="pixel-corners--wrapper mt-2 p-4 ml-auto bg-black text-white font-bold cursor-pointer w-full text-sm leading-none"
                                            on:click=move |_| on_rename_submit.run(())
                                        >
                                            "Rename field"
                                        </button>
                                    </div>
                                </Popup>
                            </Show>
                        </div>
                    </Transition>
                }
                    .into_any()
            }
        }}
    }.into_any()
}
