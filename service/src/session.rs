use crate::{db::get_cache, prelude::*};
use models::{User, UserId};
use redis::AsyncCommands;

#[derive(Serialize, Deserialize)]
struct UserSession {
    session: Session,
    user: User,
}

#[derive(Debug)]
pub struct SessionService {
    user_id: UserId,
}

impl SessionService {
    pub async fn create_session(
        insert: models::InsertSession,
        user: User,
        authentified_by_hca: bool,
    ) -> Result<(String, Session), Irror> {
        if authentified_by_hca {
            let bytes: [u8; 32] = rand::rng().random();
            let random_token = general_purpose::STANDARD.encode(bytes);
            let session = Session::from_insert(insert, random_token.clone());
            let mut con = get_cache().await.get().await?;
            let user_session = UserSession {
                session: session.clone(),
                user,
            };
            let json = serde_json::to_string(&user_session)?;
            redis::pipe()
                .set(&session.token, &json)
                .ignore()
                .expire(&session.token, 604800)
                .ignore()
                .query_async::<()>(&mut *con)
                .await?;
            Ok((random_token, session))
        } else {
            Err(Irror::Session(SessionError::NotAuthentifiedByHca))
        }
    }

    pub async fn authentify(token: &str, ip: &str, agent: &str) -> Result<User, Irror> {
        let mut con = get_cache().await.get().await?;

        let session_str: String = con
            .get(token)
            .await
            .map_err(|_| Irror::Session(SessionError::NotFound))?;

        let user_session: UserSession = serde_json::from_str(&session_str)
            .map_err(|_| Irror::Session(SessionError::ParseError))?;

        if user_session.session.ip != ip || user_session.session.user_agent != agent {
            return Err(Irror::Session(SessionError::InvalidAgentOrIp));
        }

        let record_id = user_session
            .user
            .id
            .as_ref()
            .ok_or(Irror::Session(SessionError::UserNotFoundOrDeleted))?
            .0
            .clone();
        let user: Option<User> = DB
            .query("SELECT * FROM user WHERE id = $id AND is_deleted = false")
            .bind(("id", record_id))
            .await?
            .take(0)?;
        let user = user.ok_or(Irror::Session(SessionError::UserNotFoundOrDeleted))?;

        Ok(user)
    }
}
