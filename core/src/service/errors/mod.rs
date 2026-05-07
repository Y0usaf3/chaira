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
