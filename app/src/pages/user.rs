use crate::components::{FilteredInput, Icon, IconType, PlusIcon};
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos_router::NavigateOptions;
use leptos_router::hooks::use_navigate;
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
    let update_action = Action::new(|(fname, lname): &(String, String)| {
        let first_name = fname.clone();
        let last_name = lname.clone();
        async move { update_user_info(first_name, last_name).await }
    });

    let user_data = Resource::new(
        move || update_action.version().get(),
        |_| async move { get_user_info().await },
    );

    let (first_name, set_first_name) = signal(String::new());
    let (last_name, set_last_name) = signal(String::new());
    let (validation_err, set_validation_err) = signal::<Option<String>>(None);

    Effect::new(move |_| {
        if let Some(Ok(user)) = user_data.get() {
            set_first_name.set(user.first_name.clone());
            set_last_name.set(user.last_name.clone());
            set_validation_err.set(None);
        }
    });

    let handle_save = move |ev: MouseEvent| {
        ev.prevent_default();
        let fname = first_name.get();
        let lname = last_name.get();

        if fname.is_empty() || lname.is_empty() {
            set_validation_err.set(Some("Name fields cannot be empty".to_string()));
            return;
        }

        set_validation_err.set(None);
        update_action.dispatch((fname, lname));
    };

    let naviguate = use_navigate();

    view! {
        <div class="flex h-screen w-full overflow-hidden bg-slate-100">
            <div class="order-first w-14 flex-shrink-0 flex flex-col h-full">
                <div class="flex h-14 w-full">
                    <img
                        src="/image/small_chaira.png"
                        class="h-auto w-[40px] object-contain pixelated ml-auto mt-auto"
                        alt="Chaira"
                        on:click=move |_| { naviguate("/dashboard", NavigateOptions::default()) }
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
                        <div class="border-black border-r-[3px] h-full flex flex-col">
                            <div class="w-full h-auto flex flex-row items-center justify-center px-[16px] pb-[12px] pt-[18px] bg-black text-white">
                                <Icon
                                    icon_type=IconType::NormalUser
                                    fill="#ffffff"
                                    class="!w-[25px] h-auto"
                                    extern_class="mr-[6px]"
                                />
                                <p class="leading-none">"Account"</p>
                            </div>
                        </div>
                        <Transition fallback=|| {
                            view! { <div class="p-6 text-slate-500">"Loading..."</div> }
                        }>
                            {move || match user_data.get() {
                                None => view! { <div></div> }.into_any(),
                                Some(Err(_)) => {
                                    view! {
                                        <div class="flex items-center justify-center w-full h-full">
                                            <p class="text-red-500">"Failed to load user info"</p>
                                        </div>
                                    }
                                        .into_any()
                                }
                                Some(Ok(user)) => {
                                    view! {
                                        <div class="relative flex-1 bg-white py-6 w-full h-full">
                                            <h2 class="text-2xl font-bold text-slate-800 mb-6 px-[100px]">
                                                "Account Settings"
                                            </h2>
                                            <form
                                                class="flex flex-col h-full"
                                                on:submit=move |ev| ev.prevent_default()
                                            >
                                                <div class="px-[100px] !gap-4">
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
                                                        <p class="text-slate-600 capitalize flex items-center leading-none pixel-input--wrapper p-4 !w-full">
                                                            <Icon
                                                                icon_type=match user.role() {
                                                                    UserRole::Admin => IconType::NotNormalUser,
                                                                    _ => IconType::NormalUser,
                                                                }
                                                                fill="#41566C"
                                                                class="!w-[25px] h-auto"
                                                                extern_class="mr-[6px]"
                                                            />
                                                            {user.role}
                                                        </p>
                                                    </div>
                                                </div>

                                                <button
                                                    type="submit"
                                                    disabled=move || update_action.pending().get()
                                                    class="mt-auto mr-[6px] mb-[38px] ml-auto self-end z-50 pixel-corners--wrapper p-3 bg-black text-white font-bold cursor-pointer w-auto px-8 disabled:bg-gray-500 disabled:cursor-not-allowed"
                                                    on:click=handle_save
                                                >
                                                    {move || {
                                                        if update_action.pending().get() {
                                                            "Saving..."
                                                        } else {
                                                            "Save"
                                                        }
                                                    }}
                                                </button>
                                            </form>

                                            <div class="mt-4 text-center">
                                                <Show when=move || validation_err.get().is_some()>
                                                    <p class="text-red-600">{move || validation_err.get()}</p>
                                                </Show>

                                                {move || match update_action.value().get() {
                                                    Some(Ok(_)) => {
                                                        view! {
                                                            <p class="text-green-600">"Saved successfully!"</p>
                                                        }
                                                            .into_any()
                                                    }
                                                    Some(Err(e)) => {
                                                        view! { <p class="text-red-600">{e.to_string()}</p> }
                                                            .into_any()
                                                    }
                                                    None => view! { <span></span> }.into_any(),
                                                }}
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }}
                        </Transition>
                    </div>
                </main>
            </div>
        </div>
    }
}
