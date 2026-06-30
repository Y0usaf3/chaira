use std::collections::HashMap;

use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos_router::{NavigateOptions, hooks::use_navigate};

mod column;
mod field;
mod field_popup;
mod server;
mod cell;

use server::get_table_data;

use crate::components::PlusIcon;
use crate::components::table::{field::Field, field_popup::CreateFieldPopup, server::{update_cell_value, create_table_record}};
use self::column::Column;
use self::cell::Cell;
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
        return view! {}.into_any();
    };

    let (bi, ti) = (base_id.clone(), table_id.clone());

    let on_cell_change = Callback::new(
        move |(record_id, field_id, value): (models::RecordId, models::FieldId, models::Value)| {
            let bk = bi.clone(); // dont ask me i hate this
            let tk = ti.clone();
            spawn_local(async move {
                let _ = update_cell_value(bk, tk, record_id, field_id, value).await;
            });
        },
    );

    
       let handle_create_record = {
        move |_| {
            let bk = base_id.clone();
            let tk = table_id.clone();
            spawn_local(async move {
                if let Ok(_) = create_table_record(bk, tk).await {
                    set_refresh.update(|v| *v += 1);
                }
            });
        }
    };

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
                    <div class="flex flex-col w-fit h-full overflow-visible">
                        <Suspense fallback=move || {
                            view! { <p>"Loading..."</p> }
                        }>
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
                                            let cw = col_widths.clone();
                                            move || cw.get().get(&k_for_w).copied().unwrap_or(200.0)
                                        });
                                        let or = {
                                            let set_cw = set_col_widths.clone();
                                            let k = key.clone();
                                            Callback::new(move |new_w: f64| {
                                                set_cw
                                                    .update(|map| {
                                                        map.insert(k.clone(), new_w);
                                                    });
                                            })
                                        };
                                        Some(

                                            view! {
                                                <Field field=field.clone() width=w on_resize=Some(or) />
                                            },
                                        )
                                    })
                                    .collect();
                                let field_names = fields
                                    .iter()
                                    .map(|f| f.name.clone())
                                    .collect::<Vec<_>>();
                                let field_ids: Vec<Option<FieldId>> = fields
                                    .iter()
                                    .map(|f| f.id.clone())
                                    .collect();
                                let field_configs: Vec<FieldConfig> = fields
                                    .iter()
                                    .map(|f| f.config.clone())
                                    .collect();
                                let cell_rows: Vec<_> = records
                                    .into_iter()
                                    .map({
                                        let field_keys = field_keys.clone();
                                        let field_names = field_names.clone();
                                        let field_ids = field_ids.clone();
                                        let field_configs = field_configs.clone();
                                        move |record| {
                                            let rid = record.id;
                                            let rec_cells = record.cells;
                                            let cw = col_widths.clone();
                                            let occ = on_cell_change.clone();
                                            let cells: Vec<_> = field_keys
                                                .iter()
                                                .enumerate()
                                                .filter_map({
                                                    let rid = rid.clone();
                                                    let rec_cells = rec_cells.clone();
                                                    let cw = cw.clone();
                                                    let occ = occ.clone();
                                                    let fids = field_ids.clone();
                                                    let fnames = field_names.clone();
                                                    let fconfigs = field_configs.clone();
                                                    move |(i, key)| {
                                                        let rid = rid.clone()?;
                                                        let fid = fids[i].clone()?;
                                                        let val = rec_cells
                                                            .get(&fnames[i])
                                                            .cloned()
                                                            .unwrap_or_else(|| {
                                                                Value::SingleLine(
                                                                    SingleLineValue::new(None, Some(String::new()))
                                                                        .expect("empty string is always valid"),
                                                                )
                                                            });
                                                        let cfg = fconfigs[i].clone();
                                                        let k_for_w = key.clone();
                                                        let w = Signal::derive({
                                                            let cw2 = cw.clone();
                                                            move || cw2.get().get(&k_for_w).copied().unwrap_or(200.0)
                                                        });
                                                        Some(

                                                            view! {
                                                                <Column width=w>
                                                                    <Cell
                                                                        field_config=cfg
                                                                        field_name=fid
                                                                        value=val
                                                                        on_change=occ
                                                                        record_id=rid
                                                                    />
                                                                </Column>
                                                            },
                                                        )
                                                    }
                                                })
                                                .collect();
                                            view! {
                                                <div class="flex flex-row items-center w-fit border-black border-b-[2px]">
                                                    {cells.into_view()}
                                                </div>
                                            }
                                        }
                                    })
                                    .collect();

                                view! {
                                    <div class="flex flex-col">
                                        <div class="flex flex-row items-center h-14 w-fit border-black border-b-[2px] overflow-x-auto">
                                            {headers.into_view()}
                                            <button
                                                on:click=handle_open_field_popup
                                                class="flex items-center justify-center h-full px-[7px] pt-[7px] border-black border-r-[2px] transition-colors shrink-0"
                                            >
                                                <PlusIcon class="w-[16px] h-[16px] pixelated text-black mb-auto mx-auto" />
                                            </button>
                                        </div>
                                        {cell_rows.into_view()}
                                    </div>
                                }
                                    .into_any()
                            }}
                        </Suspense>
                        <button
                            class="pl-[7px] py-[7px] border-black border-b-[2px] border-r-[2px]"
                            on:click=handle_create_record.clone()
                        >
                            <PlusIcon class="w-[16px] h-[16px] pixelated fill-black my-auto mr-auto" />
                        </button>
                        <CreateFieldPopup
                            base_id=bid
                            table_id=tid
                            show=show_field_popup
                            set_show=set_show_field_popup
                            on_created=handle_field_created
                        />
                    </div>
                }
                    .into_any()
            }
        }}
    }.into_any()
}
