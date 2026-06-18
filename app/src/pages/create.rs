use crate::components::{Footer, FilteredInput, Chira};
use leptos::{prelude::*, reactive::spawn_local};
use leptos_router::{NavigateOptions, hooks::use_navigate};
use models::Base;

#[server]
pub async fn create_base(name: String) -> Result<Base, ServerFnError> {
    let service = crate::get_authenticated_service().await?;
    let base = service
        .create_base(name)
        .await
        .map_err(|e| ServerFnError::new(format!("{e}")))?;
    Ok(base)
}

#[component]
pub fn CreatePage() -> impl IntoView {
    let (name, set_name) = signal("".to_string());
    let (description, set_description) = signal("".to_string());
    let naviguate = use_navigate();
    
    let naviguate_and_create_base = move |_| {
        spawn_local(async move {
            create_base(name.get()).await;
        });
        naviguate("/dashboard", NavigateOptions::default());
    };

    view! {
        <Chira />
        <div class="min-h-screen w-screen relative flex flex-col bg-slate-100">
            <main class="flex-grow flex flex-col justify-center items-center">

                <div class="pixel-corners--wrapper p-7 w-[400px] bg-white shadow-sm">
                    <div class="mb-6 border-b-2 border-slate-200 pb-2 border-dashed">
                        <h1 class="text-2xl font-bold text-slate-800">"Create New Base"</h1>
                        <p class="text-sm text-slate-500">"Set up your new workspace."</p>
                    </div>

                    <form class="flex flex-col gap-5" on:submit=|ev| ev.prevent_default()>

                        <FilteredInput
                            label="Base Name"
                            placeholder="e.g. OrpheusTasks"
                            value=name
                            set_value=set_name
                            filter=Callback::new(|val: String| {
                                val.chars().filter(|c| c.is_ascii_alphabetic()).collect()
                            })
                        />

                        <div class="flex flex-col gap-1">
                            <label class="text-sm font-semibold text-slate-700">
                                "Description (Optional)"
                            </label>
                            <div class="pixel-input--wrapper p-4">
                                <textarea
                                    class="placeholder:text-slate-400 focus:outline-none bg-transparent w-full resize-none"
                                    rows="3"
                                    placeholder="What are you tracking in this base?"
                                    on:input:target=move |ev| {
                                        set_description.set(ev.target().value())
                                    }
                                    prop:value=description
                                ></textarea>
                            </div>
                        </div>

                        <button
                            type="submit"
                            class="pixel-corners--wrapper mt-2 p-3 bg-slate-800 text-white font-bold hover:bg-slate-700 transition-colors cursor-pointer w-full"
                            on:click=naviguate_and_create_base
                        >
                            "Create Base"
                        </button>
                    </form>
                </div>
            </main>

            <Footer />
        </div>
    }
}
