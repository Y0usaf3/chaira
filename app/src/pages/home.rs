use crate::components::Button;
use chrono::Datelike;
use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    let year = chrono::Local::now().year();
    view! {
        <div class="h-screen w-screen relative flex justify-center items-center bg-slate-50">
            <div class="absolute top-5 right-5 flex gap-[5px]">
                <Button>"About"</Button>
                <Button>"Sign in"</Button>
            </div>

            <img src="/chaira.png" class="w-[512px] h-auto object-contain pixelated " />

            // 1.3rem for some reason :pf:
            <p class="bottom-0 left-0 absolute p-2 small-font text-[1.6rem]">{year}-{year + 1}</p>
            <div class="absolute bottom-0 left-0 h-[6px] w-full pride_gradient"></div>
        </div>
    }
}
