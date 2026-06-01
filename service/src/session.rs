use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use crate::{db::get_cache, prelude::*};
use models::{User, UserId};
use redis::AsyncCommands;

#[derive(Serialize, Deserialize)]
struct UserSession {
    session: Session,
    user: User,
}

#[derive(Clone)]
struct CachedSession {
    user: User,
    ip: String,
    agent: String,
    inserted_at: Instant,
}

const SESSION_CACHE_TTL: Duration = Duration::from_secs(5);
const SESSION_CACHE_MAX_SIZE: usize = 10_000;

static SESSION_CACHE: LazyLock<Mutex<HashMap<String, CachedSession>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

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
        {
            let cache = SESSION_CACHE.lock().unwrap();
            if let Some(entry) = cache.get(token)
                && entry.ip == ip
                && entry.agent == agent
                && entry.inserted_at.elapsed() < SESSION_CACHE_TTL
            {
                return Ok(entry.user.clone());
            }
        }

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

        {
            let mut cache = SESSION_CACHE.lock().unwrap();
            if cache.len() >= SESSION_CACHE_MAX_SIZE {
                cache.clear();
            }
            cache.insert(
                token.to_string(),
                CachedSession {
                    user: user_session.user.clone(),
                    ip: ip.to_string(),
                    agent: agent.to_string(),
                    inserted_at: Instant::now(),
                },
            );
        }

        Ok(user_session.user)
    }
}
