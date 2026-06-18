use crate::components::{FilteredInput, Popup};
use leptos::prelude::*;
use leptos::reactive::spawn_local;
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
            class="px-4 py-4 bg-slate-20 border-r-3 border-black font-medium shrink-0"
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

    let (show_create_popup, set_show_create_popup) = signal(false);
    let (table_name, set_table_name) = signal(String::new());
    let (refresh_tables, set_refresh_tables) = signal(0);

    let base_data = Resource::new(
        move || (id(), refresh_tables.get()),
        |(id, _)| async move { get_base_tables(id).await },
    );

    Effect::new(move || {
        if let Some(Err(_)) = base_data.get() {
            window().location().assign("/").unwrap();
        }
    });

    let handle_create_table = {
        let set_show = set_show_create_popup.clone();
        let set_table_name = set_table_name.clone();
        let set_refresh = set_refresh_tables.clone();
        move |_: leptos::ev::MouseEvent| {
            let name = table_name.get_untracked();
            let base_key = id();
            if name.is_empty() {
                return;
            }
            let show = set_show.clone();
            let set_table_name = set_table_name.clone();
            let set_refresh = set_refresh.clone();
            spawn_local(async move {
                if let Ok(_) = create_table(base_key.clone(), name).await {
                    show.set(false);
                    set_table_name.set(String::new());
                    set_refresh.update(|v| *v += 1);
                }
            });
        }
    };

    let show_table_selector = {
        let naviguate = naviguate.clone();
        let set_show = set_show_create_popup.clone();
        move || {
            table_id().is_empty().then(|| {
                let base_id = id();
                let naviguate = naviguate.clone();
                view! {
                    <Suspense fallback=|| {
                        view! {
                            <div class="h-14 flex-shrink-0 border-b-[3px] border-black flex items-center"></div>
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
                                            <div class="h-14 flex-shrink-0 border-b-[3px] border-black flex items-center overflow-x-auto overflow-y-hidden">
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
                                                <button
                                                    class="size-[28px] bg-black flex items-center justify-center shrink-0 pixel-corners-pfp ml-4"
                                                    on:click=move |_| set_show.set(true)
                                                >
                                                    <img
                                                        src="/svg/plus.svg"
                                                        class="size-[14px] pixelated margin-auto"
                                                    />
                                                </button>
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
        }
    };

    let main_content = move || {
        if !table_id().is_empty() {
            view! { <p class="text-lg text-slate-500">"Table: " {table_id}</p> }.into_any()
        } else {
            view! { <p class="text-lg text-slate-500">"Select a table :3"</p> }.into_any()
        }
    };

    let nav = naviguate.clone();
    let naviguate_to_dashboard = move |_| {
        nav("/dashboard", NavigateOptions::default());
    };

    view! {
        <div class="flex h-screen w-full overflow-hidden bg-slate-100">
            <div class="order-first w-14 flex-shrink-0 flex flex-col h-full border-r-[3px] border-black">
                <button class="flex h-14 w-full" on:click=naviguate_to_dashboard>
                    <img
                        src="/image/small_chaira.png"
                        class="h-auto w-[34px] object-contain pixelated m-auto"
                        alt="Chaira"
                    />
                </button>
                <div class="w-full flex justify-center mt-auto py-[4.5]">
                    <img
                        src="https://i.pinimg.com/736x/7c/41/86/7c41866499a79bca61ecf049973f5d76.jpg"
                        class="h-[40px] w-[40px] object-cover pixelated my-auto pixel-corners-pfp grayscale-75"
                    />
                </div>
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

                <Popup show=show_create_popup.into() set_show=set_show_create_popup>
                    <div class="mb-6 border-b-2 border-slate-200 pb-2 border-dashed">
                        <h2 class="text-2xl font-bold text-slate-800">"Create Table"</h2>
                        <p class="text-sm text-slate-500">"Add a new table to this base."</p>
                    </div>

                    <form class="flex flex-col gap-5" on:submit=|ev| ev.prevent_default()>
                        <FilteredInput
                            label="Table Name"
                            placeholder="e.g. Tasks"
                            value=table_name
                            set_value=set_table_name
                            filter=Callback::new(|val: String| {
                                val.chars()
                                    .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                                    .collect()
                            })
                            autofocus=true
                        />

                        <button
                            type="submit"
                            class="pixel-corners--wrapper mt-2 ml-auto p-3 bg-slate-800 text-white font-bold transition-colors cursor-pointer w-full"
                            on:click={
                                let handle = handle_create_table.clone();
                                move |ev| handle(ev)
                            }
                        >
                            "Create Table"
                        </button>
                    </form>
                </Popup>
            </div>
        </div>
    }
}
