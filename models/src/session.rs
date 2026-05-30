use crate::prelude::*;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};

// ok uh i have to modelize this for redis now

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user: UserId,
    pub token: String,
    pub ip: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: DateTime<Utc>,
}

pub struct InsertSession {
    pub user: UserId,
    pub ip: String,
    pub user_agent: String,
}

// no session patch bc we're not supposed to change this at all

impl Session {
    pub fn from_insert(insert: InsertSession, token: String) -> Self {
        let mut hasher = Sha512::new();
        hasher.update(token);
        let token = hasher.finalize();
        Session {
            id: SessionId::new(),
            token: hex::encode(token),
            ip: insert.ip,
            user_agent: insert.user_agent,
            created_at: chrono::offset::Utc::now(),
            last_used_at: chrono::offset::Utc::now(),
            user: insert.user,
        }
    }
}
