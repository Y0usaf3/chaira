use leptos::prelude::*;

#[component]
pub fn Chira() -> impl IntoView {
    view! {
        <div class="absolute top-0 left-0 h-14 w-14 z-2 flex flex-col">
            <img
                src="/image/small_chaira.png"
                class="h-auto w-[40px] object-contain pixelated mt-auto ml-auto"
                alt="Chaira"
            />
        </div>
    }
}
