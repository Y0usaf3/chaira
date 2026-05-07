use crate::models::ids::*;
use surrealdb::types::*;

// The User struct represents a global platform user.
// Users are created with the User role by default.
// Administrative privileges are granted explicitly by the system. (soon™)
// Only Administators can apply patches to Users.

#[derive(Debug, Clone, PartialEq, SurrealValue, serde::Serialize, serde::Deserialize)]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Clone, PartialEq, SurrealValue, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: Option<UserId>,
    pub created_at: Option<Datetime>,
    pub updated_at: Option<Datetime>,
    pub is_deleted: bool,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InsertUser {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserPatch {
    pub is_deleted: Option<bool>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

impl User {
    pub fn from_insert(insert: InsertUser) -> Self {
        User {
            id: None,
            created_at: None,
            updated_at: None,
            is_deleted: false,
            first_name: insert.first_name,
            last_name: insert.last_name,
            email: insert.email.to_string(),
            role: "user".to_string(),
        }
    }

    pub fn role(&self) -> UserRole {
        match self.role.as_str() {
            "admin" => UserRole::Admin,
            _ => UserRole::User,
        }
    }
}
