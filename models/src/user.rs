use crate::prelude::*;

// The User struct represents a global platform user.
// Users are created with the User role by default.
// Administrative privileges are granted explicitly by the system. (soon™)
// Only Administators can apply patches to Users.

#[derive(Debug, Clone, PartialEq, SurrealValue, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub enum UserRole {
    User,
    Admin,
}

#[derive(Debug, Clone, PartialEq, SurrealValue, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
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

    pub fn apply_patch(&mut self, patch: UserPatch) {
        if let Some(is_deleted) = patch.is_deleted {
            self.is_deleted = is_deleted
        };
        if let Some(first_name) = patch.first_name {
            self.first_name = first_name
        };
        if let Some(last_name) = patch.last_name {
            self.last_name = last_name
        };
        self.updated_at = Some(Datetime::now());
    }
}
