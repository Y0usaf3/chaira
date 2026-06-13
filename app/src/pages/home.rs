use crate::components::{Button, Footer};
use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, NavigateOptions};

#[component]
pub fn HomePage() -> impl IntoView {
    let naviguate = use_navigate();
    let naviguate_to_about = move |_| {
        naviguate("/about", NavigateOptions::default());
    };
    view! {
        <div class="min-h-screen w-screen relative flex flex-col bg-slate-50">

            <div class="absolute top-5 right-5 flex gap-[5px]">
                <Button on:click=naviguate_to_about>About</Button>
                <Button>Sign in</Button>
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
