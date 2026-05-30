use crate::prelude::*;
use models::UserId;
use redis::{AsyncCommands, JsonAsyncCommands};

#[derive(Debug)]
pub struct SessionService {
    user_id: UserId,
}

impl SessionService {
    pub async fn create_session(
        insert: models::InsertSession,
        authentified_by_hca: bool,
    ) -> Result<(String, Session), Irror> {
        if authentified_by_hca {
            let bytes: Vec<u8> = (0..32).map(|_| rand::rng().random()).collect();
            let random_token = general_purpose::STANDARD.encode(bytes);
            let session = Session::from_insert(insert, random_token.clone());
            let mut con = CACHE.get_multiplexed_async_connection().await?;
            let _: () = con.json_set(session.token.clone(), "$", &session).await?;
            // THE FUCK U MEAN "ResponseError: unknown command 'JSON.SET', with args beginning with: 'c489ff145e5ea2dbd7ca2f285e1e2943f5b4e3dd25782a8a95485e57bcf75230d9505520685194d271c2bb80ad826011dc9d5d76b1ab5c26ea389437d0cd992c'" RAHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHHH
            let _: () = con.expire(session.token.clone(), 604800).await?;
            Ok((random_token, session))
        } else {
            Err(Irror::Session(SessionError::NotAuthentifiedByHca))
        }
    }

    pub async fn authentify(token: &str, ip: &str, agent: &str) -> Result<User, Irror> {
        let mut con = CACHE.get_multiplexed_async_connection().await?;

        let session_str: String = con
            .json_get(token, "$")
            .await
            .map_err(|_| Irror::Session(SessionError::NotFound))?;

        let parsed_sessions: Vec<Session> = serde_json::from_str(&session_str)
            .map_err(|_| Irror::Session(SessionError::ParseError))?;

        let current_session = parsed_sessions
            .into_iter()
            .next()
            .ok_or(Irror::Session(SessionError::NotFound))?;

        if current_session.ip != ip || current_session.user_agent != agent {
            return Err(Irror::Session(SessionError::InvalidAgentOrIp));
        }

        let mut response = DB
            .query("SELECT * FROM user WHERE id = $user_id AND is_deleted = false")
            .bind(("user_id", current_session.user))
            .await?;

        let user: Option<User> = response.take(0)?;

        let user = user.ok_or(Irror::Session(SessionError::UserNotFoundOrDeleted))?;

        Ok(user)
    }
}
