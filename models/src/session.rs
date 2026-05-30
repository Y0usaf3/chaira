use crate::prelude::*;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

// ok uh i have to modelize this for redis now

#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub user: UserId,
    pub token: String,
    pub ip: String,
    pub user_agent: String,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>, // make surrealdb drop the session if it expires
    pub last_used_at: DateTime<Utc>,
}

#[derive(SurrealValue)]
pub struct InsertSession {
    pub user: UserId,
    pub token: String,
    pub ip: String,
    pub user_agent: String,
}

// no session patch bc we're not supposed to change this at all

impl Session {
    pub fn from_insert(insert: InsertSession) -> Self {
        Session {
            id: SessionId::new(),
            token: insert.token,
            ip: insert.ip,
            user_agent: insert.user_agent,
            created_at: chrono::offset::Utc::now(),
            expires_at: chrono::offset::Utc::now()
                .checked_add_signed(Duration::weeks(1))
                .unwrap(), // hardened expiration :p
            last_used_at: chrono::offset::Utc::now(),
            user: insert.user,
        }
    }
}
