use crate::prelude::*;

#[derive(SurrealValue, Debug)]
pub struct Session {
    pub id: Option<SessionId>,
    pub user: UserId,
    pub token: String,
    pub ip: String,
    pub user_agent: String,
    pub created_at: Option<Datetime>,
    pub expires_at: Option<Datetime>, // make surrealdb drop the session if it expires
    pub last_used_at: Option<Datetime>,
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
            id: None,
            token: insert.token,
            ip: insert.ip,
            user_agent: insert.user_agent,
            created_at: None,
            expires_at: None, // hardened expiration :p
            last_used_at: None,
            user: insert.user,
        }
    }
}
