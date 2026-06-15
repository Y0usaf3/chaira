use crate::components::Footer;
use leptos::prelude::*;

#[component]
pub fn DashboardPage() -> impl IntoView {
    view! {
        <div class="w-screen flex flex-col min-h-screen items-center h-full bg-slate-100">

            <p>"Dashboard rahh"</p>

            <Footer />

        </div>
    }
}
