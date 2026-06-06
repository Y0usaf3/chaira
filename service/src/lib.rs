pub mod base;
pub mod db;
mod encrypter;
pub mod prelude;
pub mod session;
pub mod table; // till we are done with the uh, models
pub mod user;

use crate::prelude::*;

pub static HCAUTH: LazyLock<HCAuth> = LazyLock::new(|| {
    HCAuth::new(
        env_required!("CLIENT_ID").as_str(),
        env_required!("CLIENT_SECRET").as_str(),
        env_required!("REDIRECT_URI").as_str(),
    )
});

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

// TODO: all the type checking part should be done in the models side rather than the service side
//
// simple thing to make sure the text is approved
pub fn approved(s: &str) -> Result<(), Irror> {
    if s.is_empty() || s.len() >= 30 {
        return Err(Irror::Db("Invalid length".into()));
    }

    if !s
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Irror::Db("Invalid characters".into()));
    }
    Ok(())
}
