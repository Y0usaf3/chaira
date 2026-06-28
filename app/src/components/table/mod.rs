use leptos::logging::error;
use leptos::{prelude::*, reactive::spawn_local};
use leptos_router::{NavigateOptions, hooks::use_navigate};

mod column;
mod field;
mod server;

use models::TextConfig;
use server::get_table_data;

use crate::components::table::server::create_table_field;
use crate::components::{Button, table::field::Field};

#[component]
pub fn Table(base_key: String, table_key: String) -> impl IntoView {
    let (refresh, set_refresh) = signal(0u16); // for now we are going to just refresh everytime the
    // user do smt
    let naviguate = use_navigate();
    let naviguate_to_dashboard = move || {
        naviguate("/dashboard", NavigateOptions::default());
    };

    let (base_id, table_id) = match (
        crate::parse_base_id(&base_key),
        crate::parse_table_id(&table_key),
    ) {
        (Ok(bi), Ok(ti)) => (bi, ti),
        _ => {
            naviguate_to_dashboard();
            return view! { <p class="text-red">"INVALID ID(s) !"</p> }.into_any();
        }
    };

    let base_id_clone = base_id.clone();
    let table_id_clone = table_id.clone();

    let handle_create_field = move |_: leptos::ev::MouseEvent| {
        let base_id = base_id_clone.clone();
        let table_id = table_id_clone.clone();

        spawn_local(async move {
            match create_table_field(
                base_id,
                table_id,
                "meow".to_string(),
                models::FieldConfig::Text(TextConfig::SingleLine {
                    default: None,
                    max_length: 255,
                }),
            )
            .await
            {
                Ok(_) => {
                    set_refresh.update(|v| *v += 1);
                }
                Err(e) => {
                    error!("create_table_field failed: {:?}", e);
                }
            }
        });
    };
    let data = {
        let base_id = base_id.clone();
        let table_id = table_id.clone();

        Resource::new(
            move || (base_id.clone(), table_id.clone(), refresh.get()),
            |(base, table, _)| async move { get_table_data(base, table).await },
        )
    };

    view! {
        <div class="flex flex-col w-auto h-full overflow-hidden bg-white">
            <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                <div class="flex flex-row items-center h-14 w-fit border-black border-b-[2px] border-r-[1px] overflow-x-auto">
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
            <Button on:click=handle_create_field>"create_field"</Button>
        </div>
    }
    .into_any()
}
