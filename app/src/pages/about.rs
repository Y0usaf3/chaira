use leptos::prelude::*;

use crate::components::Footer;

#[component]
pub fn AboutPage() -> impl IntoView {
    view! {
        <div class="w-screen relative flex bg-slate-50">
            <img src="/chaira.png" class="w-[480px] h-auto object-contain pixelated " />

            <Footer />
        </div>
    }
}
