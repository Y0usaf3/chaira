use app::*;
use axum::Router;
use axum::routing::get;
use axum_extra::extract::cookie::Key;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use std::net::SocketAddr;

mod oauth;
mod api;

#[tokio::main]
async fn main() {
    charac::init().await;

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let state = AppState {
        leptos_options,
        // NOTE: we only generate new keys for testing, soon we will have to store the key
        // somewhere else
        // omg and i was wondering why it was not authentificating the user even after restarting
        // the server, bruh
        key: Key::from("zgj1s9526J0/ZYEhg1AaWhy1lcM6m9XDuxNM1weGFMpiljBRRnZ5JoQGvB21EXavRniJ+HuSew7rx0gDjXQKA==".as_bytes()),
    };

    let routes = generate_route_list(App);

    let app = Router::new()
        .nest("/api", api::router())
        .route("/callback", get(oauth::oauth))
        .route("/auth", get(oauth::redirect_to_oauth))
        .leptos_routes_with_context(
            &state,
            routes,
            {
                let state = state.clone();
                move || provide_context(state.clone())
            },
            {
                let leptos_options = state.leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>({
            let options = state.leptos_options.clone();
            move |_| shell(options.clone())
        }))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
