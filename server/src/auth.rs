use axum::extract::ConnectInfo;
use axum::extract::Query;
use axum::{http::StatusCode, response::Redirect};
use axum_extra::TypedHeader;
use axum_extra::extract::PrivateCookieJar;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::cookie::SameSite;
use headers::UserAgent;
use serde::Deserialize;
use service::HCAUTH;
use service::user::UserService;
use std::net::SocketAddr;
use std::time::Instant;

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
    let start_time = Instant::now();
    println!("[OAuth] Request received. Starting execution...");

    let user_agent = user_agent
        .map(|ua| ua.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let ip = addr.ip().to_string();

    if let Some(session_cookie) = jar.get("session") {
        println!("[OAuth] Session cookie found. Attempting session validation...");
        let session_start = Instant::now();

        let session_payload = service::user::Session {
            token: session_cookie.value().to_string(),
            ip: ip.clone(),
            agent: user_agent.clone(),
        };

        match UserService::login(service::user::AuthMethod::Session(session_payload)).await {
            Ok(_service) => {
                println!(
                    "[OAuth] Existing session validated successfully. Time taken: {:?}",
                    session_start.elapsed()
                );
                println!(
                    "[OAuth] Total execution time (short-circuit): {:?}",
                    start_time.elapsed()
                );
                return Ok((jar, Redirect::to("/dashboard")));
            }
            Err(e) => {
                println!(
                    "[OAuth] Existing session validation failed/expired after {:?}. Error: {:?}",
                    session_start.elapsed(),
                    e
                );
            }
        }
    } else {
        println!(
            "[OAuth] No active session cookie found. Proceeding with standard OAuth exchange."
        );
    }

    println!("[OAuth] Exchanging authorization code for user login...");
    let oauth_login_start = Instant::now();
    let hca_auth = service::user::AuthMethod::Hca(params.code.clone());

    let mut service = match UserService::login(hca_auth).await {
        Ok(existing_service) => {
            println!(
                "[OAuth] OAuth authentication successful. Time taken: {:?}",
                oauth_login_start.elapsed()
            );
            existing_service
        }
        Err(e) => {
            eprintln!(
                "[OAuth] Registration/Login error after {:?}: {:?}",
                oauth_login_start.elapsed(),
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    println!("[OAuth] Creating new user session in data store...");
    let create_session_start = Instant::now();

    let session_token = match service.create_session(ip, user_agent).await {
        Ok(token) => {
            println!(
                "[OAuth] Session created successfully. Time taken: {:?}",
                create_session_start.elapsed()
            );
            token
        }
        Err(e) => {
            eprintln!(
                "[OAuth] Failed to create session after {:?}: {:?}",
                create_session_start.elapsed(),
                e
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let cookie = Cookie::build(("session", session_token))
        .path("/")
        .max_age(time::Duration::days(5))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

    let updated_jar = jar.add(cookie);

    println!(
        "[OAuth] Process completed successfully. Total handling time: {:?}",
        start_time.elapsed()
    );
    Ok((updated_jar, Redirect::to("/dashboard")))
}

pub async fn redirect_to_oauth() -> Result<Redirect, StatusCode> {
    Ok(Redirect::to(
        &HCAUTH.get_oauth_uri(&["openid", "profile", "email", "name"]),
    ))
}
