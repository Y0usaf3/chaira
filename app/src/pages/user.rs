use crate::components::{FilteredInput, Icon, IconType, PlusIcon};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::reactive::spawn_local;
use models::{User, UserPatch, UserRole};

#[server]
pub async fn get_user_info() -> Result<User, ServerFnError> {
    let mut service = crate::get_authenticated_service().await?;
    let user = service
        .user()
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to get user: {e:?}")))?;
    Ok(user)
}

#[server]
pub async fn update_user_info(
    first_name: String,
    last_name: String,
) -> Result<User, ServerFnError> {
    let mut service = crate::get_authenticated_service().await?;
    let patch = UserPatch {
        is_deleted: None,
        first_name: Some(first_name),
        last_name: Some(last_name),
    };
    let user = service
        .update_self_user(patch)
        .await
        .map_err(|e| ServerFnError::new(format!("Failed to update user: {e:?}")))?;
    Ok(user)
}

#[component]
pub fn UserPage() -> impl IntoView {
    let (refresh_count, set_refresh_count) = signal(0);
    let (save_status, set_save_status) = signal::<Option<Result<(), String>>>(None);
    let (first_name, set_first_name) = signal(String::new());
    let (last_name, set_last_name) = signal(String::new());
    let initialized = RwSignal::new(false);

    let user_resource = Resource::new(
        move || refresh_count.get(),
        |_| async move { get_user_info().await },
    );

    Effect::new(move |_| {
        if let Some(Ok(ref user)) = user_resource.get() {
            if !initialized.get() {
                set_first_name.set(user.first_name.clone());
                set_last_name.set(user.last_name.clone());
                initialized.set(true);
            }
        }
    });

    let handle_save = move |_: MouseEvent| {
        let fname = first_name.get_untracked();
        let lname = last_name.get_untracked();
        if fname.is_empty() || lname.is_empty() {
            set_save_status.set(Some(Err("Name fields cannot be empty".to_string())));
            return;
        }
        spawn_local(async move {
            match update_user_info(fname, lname).await {
                Ok(_) => {
                    initialized.set(false);
                    set_save_status.set(Some(Ok(())));
                    set_refresh_count.update(|v| *v += 1);
                }
                Err(e) => {
                    set_save_status.set(Some(Err(e.to_string())));
                }
            }
        });
    };

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
                    <button class="pixel-corners-pfp bg-black w-[32px] h-[32px] flex items-center justify-center">
                        <PlusIcon class="w-[16px] h-[16px] pixelated text-white" />
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
                    <div class="overflow-none w-full h-full flex flex-row">
                        <div class="border-black border-r-[3px] w-[64px] h-full hover:w-[128px] ease-linear transition-all duration-80"></div>
                        <Suspense fallback=|| {
                            view! { <p>"Loading..."</p> }
                        }>
                            {move || {
                                Suspend::new(async move {
                                    match user_resource.get() {
                                        Some(Ok(user)) => {
                                            view! {
                                                <div class="bg-white p-6 w-full">
                                                    <h2 class="text-2xl font-bold text-slate-800 mb-6">
                                                        "Account Settings"
                                                    </h2>
                                                    <form
                                                        class="flex flex-col gap-4"
                                                        on:submit=|ev| ev.prevent_default()
                                                    >
                                                        <FilteredInput
                                                            label="First Name"
                                                            placeholder="First name"
                                                            value=first_name
                                                            set_value=set_first_name
                                                            filter=Callback::new(|val: String| val)
                                                            autofocus=false
                                                        />

                                                        <FilteredInput
                                                            label="Last Name"
                                                            placeholder="Last name"
                                                            value=last_name
                                                            set_value=set_last_name
                                                            filter=Callback::new(|val: String| val)
                                                            autofocus=false
                                                        />

                                                        <div class="flex flex-col gap-1">
                                                            <label class="text-sm font-semibold text-slate-700">
                                                                "Email"
                                                            </label>
                                                            <div class="pixel-input--wrapper p-4 !w-full">
                                                                <input
                                                                    type="text"
                                                                    class="placeholder:text-slate-400 focus:outline-none bg-transparent w-full text-slate-500"
                                                                    value=user.email.clone()
                                                                    disabled=true
                                                                />
                                                            </div>
                                                        </div>

                                                        <div class="flex flex-col gap-1">
                                                            <label class="text-sm font-semibold text-slate-700">
                                                                "Role"
                                                            </label>
                                                            <p class="text-slate-600 capitalize px-1">
                                                                {match user.role() {
                                                                    UserRole::Admin => {
                                                                        view! {
                                                                            <Icon
                                                                                icon_type=IconType::NotNormalUser
                                                                                fill="#000000"
                                                                                class="!w-[25px] h-auto"
                                                                                extern_class="mr-[4px]"
                                                                            />
                                                                        }
                                                                            .into_any()
                                                                    }
                                                                    _ => {

                                                                        view! {
                                                                            <Icon
                                                                                icon_type=IconType::NormalUser
                                                                                fill="#000000"
                                                                                class="!w-[25px] h-auto"
                                                                                extern_class="mr-[4px]"
                                                                            />
                                                                        }
                                                                            .into_any()
                                                                    }
                                                                }} {user.role}
                                                            </p>
                                                        </div>

                                                        <button
                                                            type="submit"
                                                            class="pixel-corners--wrapper mt-2 p-3 bg-black text-white font-bold transition-colors cursor-pointer w-full"
                                                            on:click=handle_save
                                                        >
                                                            "Save Changes"
                                                        </button>
                                                    </form>

                                                    {move || {
                                                        save_status
                                                            .get()
                                                            .map(|status| match status {
                                                                Ok(()) => {
                                                                    view! {
                                                                        <p class="text-green-600 mt-2 text-center">
                                                                            "Saved successfully!"
                                                                        </p>
                                                                    }
                                                                        .into_any()
                                                                }
                                                                Err(msg) => {
                                                                    view! { <p class="text-red-600 mt-2 text-center">{msg}</p> }
                                                                        .into_any()
                                                                }
                                                            })
                                                    }}
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        Some(Err(_)) => {

                                            view! {
                                                <div class="flex items-center justify-center w-full h-full">
                                                    <p class="text-red-500">"Failed to load user info"</p>
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        _ => {
                                            view! {
                                                <div class="flex items-center justify-center w-full h-full">
                                                    <p class="text-slate-500">"Loading..."</p>
                                                </div>
                                            }
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
