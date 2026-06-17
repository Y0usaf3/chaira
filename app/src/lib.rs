use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes, ProtectedParentRoute, Outlet},
    path,
};

mod components;
mod pages;

use pages::*;

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

#[server]
pub async fn is_athenticated() -> Result<bool, ServerFnError> {
   match crate::get_authenticated_service().await {
        Ok(_) => Ok(true),
        Err(_) => Err(ServerFnError::Registration("whatever".to_string()))
    }
}

#[component]
pub fn App() -> impl IntoView {
    let auth_status = Resource::new(
        || (), 
        |_| async move {
            is_athenticated().await.is_ok()
        }
    );
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/chaira.css" />
        <Title text="Chaira" />
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                    <Route path=path!("/about") view=AboutPage />
                    <ProtectedParentRoute
                        path=path!("")
                        view=|| view! { <Outlet /> }
                        condition=move || auth_status.get()
                        redirect_path=|| "/"
                    >
                        <Route path=path!("/dashboard") view=DashboardPage />
                        <Route path=path!("/create") view=CreatePage />
                    </ProtectedParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

#[cfg(feature = "ssr")]
pub async fn get_authenticated_service() -> Result<service::user::UserService, ServerFnError> {
    use axum::extract::ConnectInfo;
    use axum::http::HeaderMap;
    use axum_extra::extract::cookie::PrivateCookieJar;
    use leptos::context::use_context;
    use leptos::prelude::ServerFnError;
    use leptos_axum::{extract, extract_with_state};
    use service::user::{AuthMethod, Session, UserService};
    use std::net::SocketAddr;
    let ConnectInfo(addr): ConnectInfo<SocketAddr> = extract()
        .await
        .map_err(|e| ServerFnError::new(format!("Connection info missing: {e:?}")))?;

    let headers: HeaderMap = extract()
        .await
        .map_err(|e| ServerFnError::new(format!("Headers missing: {e:?}")))?;

    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let state = use_context::<crate::AppState>()
        .ok_or_else(|| ServerFnError::new("AppState not found in context"))?;

    let jar = extract_with_state::<PrivateCookieJar, crate::AppState>(&state)
        .await
        .map_err(|e| ServerFnError::new(format!("Cookie extraction failed: {e:?}")))?;

    let secret = jar
        .get("session")
        .map(|c| c.value().to_string())
        .ok_or_else(|| ServerFnError::new("No session cookie found"))?;

    let service = UserService::login(AuthMethod::Session(Session {
        token: secret,
        ip: addr.ip().to_string(),
        agent: user_agent.to_string(),
    }))
    .await
    .map_err(|e| ServerFnError::new(format!("Authentication failed: {e:?}")))?;

    Ok(service)
}
