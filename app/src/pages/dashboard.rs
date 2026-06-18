use std::time::Instant;

use crate::components::BaseCard;
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};
use models::Base;

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

#[component]
pub fn DashboardPage() -> impl IntoView {
    let naviguate = use_navigate();
    let naviguate_to_create_base = move |_| {
        naviguate("/create", NavigateOptions::default());
    };

    let (refresh_count, set_refresh_count) = signal(0);
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
                        on:click=naviguate_to_create_base
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
    }
}
