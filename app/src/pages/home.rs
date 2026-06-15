use crate::components::{Button, Footer};
use leptos::prelude::*;
use leptos_router::{NavigateOptions, hooks::use_navigate};

#[component]
pub fn HomePage() -> impl IntoView {
    let naviguate = use_navigate();
    let naviguate_to_about = move |_| {
        naviguate("/about", NavigateOptions::default());
    };
    let naviguate_to_auth = move |_| {
        let _ = window().location().assign("/auth");
    };

    view! {
        <div class="min-h-screen w-screen relative flex flex-col bg-slate-100">

            <div class="absolute top-5 right-5 flex gap-[5px]">
                <Button on:click=naviguate_to_about>"About"</Button>
                <Button on:click=naviguate_to_auth>"Sign in"</Button>
            </div>

            <main class="flex-grow flex flex-col justify-center items-center">
                <img
                    src="/chaira.png"
                    class="w-[512px] h-auto object-contain pixelated"
                    alt="Chaira"
                />
            </main>

            <Footer />

        </div>
    }
}
