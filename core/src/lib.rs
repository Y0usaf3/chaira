pub mod models;

#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod service;

#[cfg(feature = "ssr")]
pub use db::init;
#[cfg(feature = "ssr")]
pub use dotenvy;

#[cfg(feature = "ssr")]
use chacha20poly1305::Key;
#[cfg(feature = "ssr")]
use hackclub_auth_api::HCAuth;
#[cfg(feature = "ssr")]
use std::sync::LazyLock;

#[cfg(feature = "ssr")]
pub static HCAUTH: LazyLock<HCAuth> = LazyLock::new(|| {
    HCAuth::new(
        env_required!("CLIENT_ID").as_str(),
        env_required!("CLIENT_SECRET").as_str(),
        env_required!("REDIRECT_URI").as_str(),
    )
});

#[cfg(feature = "ssr")]
pub static MASTER_KEY: LazyLock<Key> = LazyLock::new(|| {
    let key_hex = env_required!("MASTER_KEY");
    let key_bytes = hex::decode(&key_hex).expect("MASTER_KEY must be valid hex string");
    if key_bytes.len() != 32 {
        panic!("MASTER_KEY must be exactly 32 bytes (64 hex characters)");
    }
    let mut key_array = [0u8; 32];
    key_array.copy_from_slice(&key_bytes);
    Key::from(key_array)
});
