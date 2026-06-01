use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chacha20poly1305::Error as EncryptionErr;
use redis::RedisError as RedisErr;
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid or expired authentication token")]
    InvalidToken,

    #[error("Failed to verify authentication credentials")]
    VerificationFailed,

    #[error("Session token does not exist or has expired")]
    SessionNotFound,
}

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("You have to be authentified by HackClub Auth to be able to create new sessions")]
    NotAuthentifiedByHca,

    #[error("Session not found or has expired")]
    NotFound,

    #[error("Failed to parse session data from the cache")]
    ParseError,

    #[error("Session invalid: IP or User-Agent mismatch. Possible session theft detected!")]
    InvalidAgentOrIp,

    #[error("Session has expired")]
    Expired,

    #[error("The user associated with this session does not exist or has been deleted")]
    UserNotFoundOrDeleted,
}

#[derive(Error, Debug)]
pub enum UserError {
    #[error("User does not exist")]
    NotFound,

    #[error("User account has been deleted")]
    Deleted,

    #[error("Failed to update user: {0}")]
    UpdateFailed(String),

    #[error("Cannot perform this operation on yourself")]
    CannotActionSelf,
}

#[derive(Error, Debug)]
pub enum PermissionError {
    #[error("Insufficient permissions for this operation")]
    Insufficient,

    #[error("Admin role required")]
    AdminRequired,
}

#[derive(Error, Debug)]
pub enum BaseError {
    #[error("Base not found or access denied")]
    NotFound,

    #[error("Failed to create base")]
    CreateFailed,

    #[error("Failed to delete base")]
    DeleteFailed,
}

#[derive(Error, Debug)]
pub enum TableError {
    #[error("Table not found or access denied")]
    NotFound,

    #[error("Failed to create table")]
    CreateFailed,

    #[error("Failed to delete table")]
    DeleteFailed,

    #[error("This action is unauthorized")]
    Unauthorized,

    #[error("Update failed")]
    UpdateFailed,
}

#[derive(Error, Debug)]
pub enum EncryptionError {
    #[error("Encryption failed")]
    EncryptionFailed,

    #[error("Decryption failed")]
    DecryptionFailed,

    #[error("Invalid nonce")]
    InvalidNonce,
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Internal database query failed: {0}")]
    QueryFailed(String),

    #[error("Transaction failed to commit: {0}")]
    TransactionFailed(String),
}

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Token not found or access denied")]
    NotFound,

    #[error("Failed to create table")]
    CreateFailed,

    #[error("Failed to delete table")]
    DeleteFailed,

    #[error("This action is unauthorized")]
    Unauthorized,
}

#[derive(Error, Debug)]
pub enum RedisError {
    #[error("Failed to connect to Redis")]
    ConnectionFailed,

    #[error("Redis command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to retrieve value from Redis")]
    GetFailed,

    #[error("Failed to store value in Redis")]
    SetFailed,

    #[error("Failed to delete value from Redis")]
    DeleteFailed,

    #[error("Redis key not found")]
    KeyNotFound,

    #[error("Redis operation timed out")]
    Timeout,

    #[error("Failed to serialize data for Redis")]
    SerializationFailed,

    #[error("Failed to deserialize data from Redis")]
    DeserializationFailed,
}

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

    #[error("redis error: {0}")]
    Redis(#[from] RedisError),

    #[error("couldnt serialize")]
    Serialization,

    #[error("Session error: {0}")]
    Session(SessionError),
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

impl From<RedisErr> for Irror {
    fn from(error: RedisErr) -> Self {
        eprintln!("{error:?}");
        Self::Redis(RedisError::CommandFailed(error.to_string()))
    }
}

impl From<bb8_redis::bb8::RunError<redis::RedisError>> for Irror {
    fn from(error: bb8_redis::bb8::RunError<redis::RedisError>) -> Self {
        eprintln!("{error:?}");
        match error {
            bb8_redis::bb8::RunError::User(redis_err) => {
                Self::Redis(RedisError::CommandFailed(redis_err.to_string()))
            }
            bb8_redis::bb8::RunError::TimedOut => Self::Redis(RedisError::CommandFailed(
                "Redis pool connection timed out".to_string(),
            )),
        }
    }
}
