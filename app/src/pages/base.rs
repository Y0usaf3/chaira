use crate::components::Button;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::{use_navigate, use_params_map};
use models::surrealdb_types::RecordId;
use models::{BaseId, Table, ToSql};

#[server]
pub async fn get_base_tables(base_key: String) -> Result<Vec<Table>, ServerFnError> {
    let mut service = crate::get_authenticated_service().await?;
    let rid = RecordId::parse_simple(&format!("base:{base_key}"))
        .map_err(|e| ServerFnError::new(format!("Invalid base id: {e:?}")))?;
    let base_id = BaseId(rid);

    service
        .open_base(base_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to open base: {e:?}")))?;

    let tables = service
        .current_base
        .as_mut()
        .ok_or_else(|| ServerFnError::new("No base service available"))?
        .list_tables()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to list tables: {e:?}")))?;

    Ok(tables)
}

#[server]
pub async fn create_table(base_key: String, name: String) -> Result<Table, ServerFnError> {
    let mut service = crate::get_authenticated_service().await?;
    let rid = RecordId::parse_simple(&format!("base:{base_key}"))
        .map_err(|e| ServerFnError::new(format!("Invalid base id: {e:?}")))?;
    let base_id = BaseId(rid);
    service
        .open_base(base_id)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to open base: {e:?}")))?;
    let table = service
        .current_base
        .as_mut()
        .ok_or_else(|| ServerFnError::new("No base service available"))?
        .create_table(name)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to list tables: {e:?}")))?;
    Ok(table)
}

#[component]
fn TableButton<F>(table: Table, base_id: String, naviguate: F) -> impl IntoView
where
    F: Fn(&str, NavigateOptions) + Clone + 'static,
{
    let key = table.id.map(|id| id.0.key.to_sql()).unwrap_or_default();
    let name = table.name;
    let path = format!("/base/{}/{}", base_id, key);

    view! {
        <button
            class="px-4 py-1.5 bg-white border-2 border-black pixel-corners--wrapper font-medium hover:bg-slate-200 shrink-0"
            on:click=move |_| { naviguate(&path, NavigateOptions::default()) }
        >
            {name}
        </button>
    }
}

#[component]
pub fn BasePage() -> impl IntoView {
    let params = use_params_map();
    let naviguate = use_navigate();

    let id = move || {
        params
            .read()
            .get("id")
            .map(|s| s.clone())
            .unwrap_or_else(|| "404".to_string())
    };
    let table_id = move || {
        params
            .read()
            .get("table_id")
            .map(|s| s.clone())
            .unwrap_or_default()
    };

    let base_data = Resource::new(move || id(), |id| async move { get_base_tables(id).await });

    Effect::new(move || {
        if let Some(Err(_)) = base_data.get() {
            window().location().assign("/").unwrap();
        }
    });

    let show_table_selector = move || {
        table_id().is_empty().then(|| {
            let base_id = id();
            let naviguate = naviguate.clone();
            view! {
                <Suspense fallback=|| {
                    view! {
                        <div class="h-14 flex-shrink-0 border-b-[3px] border-black flex items-center px-4 gap-4"></div>
                    }
                }>
                    {move || {
                        let base_id = base_id.clone();
                        let naviguate = naviguate.clone();
                        let base_data = base_data.clone();
                        Suspend::new(async move {
                            match base_data.get() {
                                Some(Ok(tables)) => {
                                    view! {
                                        <div class="h-14 flex-shrink-0 border-b-[3px] border-black flex items-center px-4 gap-4 overflow-x-auto">
                                            {tables
                                                .into_iter()
                                                .map(move |table| {
                                                    view! {
                                                        <TableButton
                                                            table=table
                                                            base_id=base_id.clone()
                                                            naviguate=naviguate.clone()
                                                        />
                                                    }
                                                })
                                                .collect_view()}
                                        </div>
                                    }
                                        .into_any()
                                }
                                _ => view! {}.into_any(),
                            }
                        })
                    }}
                </Suspense>
            }
        })
    };

    let main_content = move || {
        if !table_id().is_empty() {
            view! { <p class="text-lg text-slate-500">"Table: " {table_id}</p> }.into_any()
        } else {
            view! { <p class="text-lg text-slate-500">"Select a table :3"</p> }.into_any()
        }
    };

    view! {
        <div class="flex h-screen w-full overflow-hidden bg-slate-100">
            <div class="order-first w-14 flex-shrink-0 flex flex-col h-full border-r-[3px] border-black">
                <div class="flex h-14 w-full">
                    <img
                        src="/image/small_chaira.png"
                        class="h-auto w-[34px] object-contain pixelated m-auto"
                        alt="Chaira"
                    />
                </div>
                <div class="w-full flex justify-center mt-auto py-[4.5]"></div>
            </div>

            <div class="flex flex-1 flex-col overflow-hidden">
                <div class="h-14 flex-shrink-0 border-b-[3px] border-black flex items-stretch">
                    <h1 class="text-[0.9rem] font-bold self-center ml-4">{id}</h1>
                    <div class="ml-auto flex items-stretch self-center py-4">
                        <button class="h-full py-4 uppercase border-black border-l-[3px] px-2">
                            "data"
                        </button>
                        <button class="h-full py-4 uppercase border-black border-l-[3px] px-2">
                            "automations"
                        </button>
                    </div>
                </div>

                {show_table_selector}

                <div class="flex-1 overflow-hidden p-6">{main_content}</div>
            </div>
        </div>
    }
}
