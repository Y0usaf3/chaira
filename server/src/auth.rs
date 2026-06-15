use axum::extract::ConnectInfo;
use axum::extract::Query;
use axum::{http::StatusCode, response::Redirect};
use axum_extra::TypedHeader;
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use headers::UserAgent;
use leptos::logging::*;
use serde::Deserialize;
use service::HCAUTH;
use service::user::UserService;
use std::net::SocketAddr;

#[derive(Deserialize)]
pub struct Code {
    code: String,
}

pub async fn oauth(
    Query(params): Query<Code>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    jar: PrivateCookieJar,
    user_agent: Option<TypedHeader<UserAgent>>,
) -> Result<(PrivateCookieJar, Redirect), StatusCode> {
    let user_agent = user_agent
        .map(|ua| ua.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let ip = addr.ip().to_string();

    if let Some(session_cookie) = jar.get("session") {
        let session_payload = service::user::Session {
            token: session_cookie.value().to_string(),
            ip: ip.clone(),
            agent: user_agent.clone(),
        };

        if let Ok(_service) =
            UserService::login(service::user::AuthMethod::Session(session_payload)).await
        {
            return Ok((jar, Redirect::to("/dashboard")));
        }
    }

    let hca_auth = service::user::AuthMethod::Hca(params.code.clone());

    let mut service = match UserService::login(hca_auth).await {
        Ok(existing_service) => existing_service,
        Err(e) =>  {
            eprintln!("Registration error: {:?}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    };


    let session_token = service
        .create_session(ip, user_agent)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;


    let cookie = Cookie::build(("session", session_token))
        .path("/")
        .max_age(time::Duration::days(5))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    let updated_jar = jar.add(cookie);

    Ok((updated_jar, Redirect::to("/dashboard")))
}

pub async fn redirect_to_oauth() -> Result<Redirect, StatusCode> {
    Ok(Redirect::to(
        &HCAUTH.get_oauth_uri(&["openid", "profile", "email", "name"]),
    ))
}
