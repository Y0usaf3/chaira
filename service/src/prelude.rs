pub use std::sync::LazyLock;
pub use std::time::{Duration, SystemTime};

pub use base64::{Engine, engine::general_purpose};
pub use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
pub use dotenvy;
pub use hackclub_auth_api::HCAuth;
pub use rand::RngExt;
pub use serde::{Deserialize, Serialize};
pub use service_macros::requires;
pub use surrealdb::{
    opt::PatchOp,
    types::{Datetime, SurrealValue},
};

pub use crate::{
    HCAUTH, MASTER_KEY, approved,
    db::{CACHE, DB, error::*},
    encrypter::*,
};
pub use models::*;
