use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};

mod column;
mod field;
mod field_popup;
mod server;

use server::get_table_data;

use crate::components::table::{field::Field, field_popup::CreateFieldPopup};
use crate::components::{PlusIcon};

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
                    <div class="flex flex-col w-auto h-full overflow-visible bg-white">
                        <div class="flex flex-row items-center">
                            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                                <div class="flex flex-row items-center h-14 w-fit border-black border-b-[2px] overflow-x-auto">
                                    {move || {
                                        let data = data.get();
                                        let data_value = match data {
                                            Some(Ok(value)) => value,
                                            _ => (vec![], vec![]),
                                        };
                                        data_value
                                            .0
                                            .into_iter()
                                            .map(|field| view! { <Field field=field /> })
                                            .collect::<Vec<_>>()
                                            .into_any()
                                    }}
                                </div>
                            </Suspense>
                            <button
                                on:click=handle_open_field_popup
                                class="flex items-center justify-center h-full px-3 border-black border-r-[2px] transition-colors"
                            >
                                <PlusIcon class="w-[16px] h-[16px] pixelated text-black mb-auto mx-auto mt-[8px]" />
                            </button>
                        </div>
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
    }
}
