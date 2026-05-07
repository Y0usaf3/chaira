mod components;
mod pages;
mod utils;

#[cfg(feature = "ssr")]
pub use utils::get_authenticated_service;

use crate::pages::*;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::path;
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

#[derive(Clone)]
pub struct AppState {
    #[cfg(feature = "ssr")]
    pub leptos_options: LeptosOptions,
    #[cfg(feature = "ssr")]
    pub key: axum_extra::extract::cookie::Key,
}

#[cfg(feature = "ssr")]
impl axum::extract::FromRef<AppState> for axum_extra::extract::cookie::Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[cfg(feature = "ssr")]
impl axum::extract::FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/chara.css" />

        <Title text="Chaira" />

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                    <Route path=path!("/dashboard") view=DashboardPage />
                    <Route path=path!("/base/:id") view=BasePage />
                </Routes>
            </main>
        </Router>
    }
}
