use crate::prelude::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl Session {
    pub fn from_insert(insert: InsertSession, token: String) -> Self {
        Session {
            id: SessionId::new(),
            token,
            ip: insert.ip,
            user_agent: insert.user_agent,
            created_at: chrono::offset::Utc::now(),
            last_used_at: chrono::offset::Utc::now(),
            user: insert.user,
        }
    }
}
