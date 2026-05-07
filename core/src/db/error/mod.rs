use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chacha20poly1305::Error as EncryptionErr;
use serde_json::json;
use thiserror::Error;

use crate::service::errors::{
    ApiError, AuthError, BaseError, DatabaseError, EncryptionError, PermissionError, TableError,
    UserError,
};

#[derive(Error, Debug)]
pub enum Irror {
    #[error("database error: {0}")]
    Db(String),

    #[error("authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("user error: {0}")]
    User(#[from] UserError),

    #[error("permission error: {0}")]
    Permission(#[from] PermissionError),

    #[error("encryption error: {0}")]
    Encryption(#[from] EncryptionError),

    #[error("database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("base error: {0}")]
    Base(#[from] BaseError),

    #[error("table error: {0}")]
    Table(#[from] TableError),

    #[error("api error: {0}")]
    Api(#[from] ApiError),
}

impl IntoResponse for Irror {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            Self::Auth(_) => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::Permission(_) => (StatusCode::FORBIDDEN, self.to_string()),
            Self::User(UserError::NotFound)
            | Self::Base(BaseError::NotFound)
            | Self::Table(TableError::NotFound) => (StatusCode::NOT_FOUND, self.to_string()),
            Self::User(UserError::CannotActionSelf) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({
            "error": message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl From<surrealdb::Error> for Irror {
    fn from(error: surrealdb::Error) -> Self {
        eprintln!("{error:?}");
        Self::Db(error.to_string())
    }
}

impl From<EncryptionErr> for Irror {
    fn from(error: EncryptionErr) -> Self {
        eprintln!("{error:?}");
        Self::Encryption(EncryptionError::EncryptionFailed)
    }
}
