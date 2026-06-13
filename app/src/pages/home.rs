use crate::components::{Button, Footer};
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="h-screen w-screen relative flex justify-center items-center bg-slate-50">
            <div class="absolute top-5 right-5 flex gap-[5px]">
                <Button>"About"</Button>
                <Button>"Sign in"</Button>
            </div>

            <img src="/chaira.png" class="w-[512px] h-auto object-contain pixelated " />
            <Footer />
        </div>
    }
}
