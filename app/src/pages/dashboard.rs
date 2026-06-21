use std::time::Instant;

use crate::components::{BaseCard, FilteredInput, Popup};
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use models::{Base, ToSql};

#[server]
pub async fn get_user_bases() -> Result<Vec<Base>, ServerFnError> {
    let instant = Instant::now();
    let service = crate::get_authenticated_service().await?;

    let bases = service
        .list_bases()
        .await
        .map_err(|e| ServerFnError::new(format!("Listing Bases failed: {e:?}")))?;
    println!("{:?}", instant.elapsed());
    Ok(bases)
}

#[server]
pub async fn create_base_dash(name: String) -> Result<Base, ServerFnError> {
    let service = crate::get_authenticated_service().await?;
    let base = service
        .create_base(name)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    Ok(base)
}

#[component]
pub fn DashboardPage() -> impl IntoView {
    let naviguate = use_navigate();
    let (show_create_popup, set_show_create_popup) = signal(false);
    let (name, set_name) = signal(String::new());

    let (refresh_count, set_refresh_count) = signal(0);

    let handle_create_base = {
        let set_show = set_show_create_popup.clone();
        let set_name = set_name.clone();
        let set_refresh = set_refresh_count.clone();
        move |_: leptos::ev::MouseEvent| {
            let base_name = name.get_untracked();
            if base_name.is_empty() {
                return;
            }
            let show = set_show.clone();
            let set_name = set_name.clone();
            let set_refresh = set_refresh.clone();
            spawn_local(async move {
                if let Ok(_) = create_base_dash(base_name).await {
                    show.set(false);
                    set_name.set(String::new());
                    set_refresh.update(|v| *v += 1);
                }
            });
        }
    };
    let bases = Resource::new(
        move || refresh_count.get(),
        |_| async move { get_user_bases().await },
    );

    Effect::new(move || {
        if let Some(Err(_)) = bases.get() {
            window().location().assign("/").unwrap();
        }
    });

    view! {
        <div class="flex h-screen w-full overflow-hidden bg-slate-100">
            <div class="order-first w-14 flex-shrink-0 flex flex-col h-full">
                <div class="flex h-14 w-full">
                    <img
                        src="/image/small_chaira.png"
                        class="h-auto w-[40px] object-contain pixelated ml-auto mt-auto"
                        alt="Chaira"
                    />
                </div>

                <div class="w-full flex justify-center mt-auto py-[4.5]">
                    <button
                        on:click=move |_| set_show_create_popup.set(true)
                        class="pixel-corners-pfp bg-black w-[32px] h-[32px] flex items-center justify-center"
                    >
                        <img src="/svg/plus.svg" class="w-[16px] h-[16px] pixelated fill-white" />
                    </button>
                </div>
            </div>

            <div class="flex flex-1 flex-col overflow-hidden">
                <div class="h-14 flex-shrink-0">
                    <div class="flex h-full w-full items-center justify-end px-[4.5]">
                        <img
                            src="https://i.pinimg.com/736x/7c/41/86/7c41866499a79bca61ecf049973f5d76.jpg"
                            class="h-[40px] w-[40px] object-cover pixelated my-auto pixel-corners-pfp grayscale-75"
                        />
                    </div>
                </div>
                <main class="pixel-corners-wrapper flex-1 overflow-hidden mb-[-3px] mr-[-3px] bg-slate-50">
                    <div class="overflow-y-auto w-full h-full p-6">
                        <Suspense>
                            {move || {
                                Suspend::new(async move {
                                    match bases.get() {
                                        Some(Ok(list)) if list.is_empty() => {
                                            view! { <p>"EMPTY"</p> }.into_any()
                                        }
                                        Some(Ok(list)) => {
                                            view! {
                                                <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 w-full justify-items-stretch">
                                                    {list
                                                        .into_iter()
                                                        .map(|base| {
                                                            view! { <BaseCard base=base.clone() /> }
                                                        })
                                                        .collect_view()}
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        Some(Err(_)) => {
                                            view! {
                                                <p class="text-red-500">
                                                    "Unauthentified ! you silly goober"
                                                </p>
                                            }
                                                .into_any()
                                        }
                                        _ => {
                                            view! { <p class="text-red-500">"Unknown error"</p> }
                                                .into_any()
                                        }
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </main>
            </div>
        </div>

        <Popup show=show_create_popup.into() set_show=set_show_create_popup>
            <div class="mb-6 border-b-2 border-slate-200 pb-2 border-dashed">
                <h2 class="text-2xl font-bold text-slate-800">"Create New Base"</h2>
                <p class="text-sm text-slate-500">"Set up your new workspace."</p>
            </div>

            <form class="flex flex-col gap-5" on:submit=|ev| ev.prevent_default()>
                <FilteredInput
                    label="Base Name"
                    placeholder="e.g. OrpheusTasks"
                    value=name
                    set_value=set_name
                    filter=Callback::new(|val: String| {
                        val.chars()
                            .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                            .collect()
                    })
                    autofocus=true
                />

                <button
                    type="submit"
                    class="pixel-corners--wrapper mt-2 p-3 bg-slate-800 text-white font-bold hover:bg-slate-700 transition-colors cursor-pointer w-full"
                    on:click={
                        let handle = handle_create_base.clone();
                        move |ev| handle(ev)
                    }
                >
                    "Create Base"
                </button>
            </form>
        </Popup>
    }
}
