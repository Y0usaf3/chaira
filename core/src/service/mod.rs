use crate::db::Irror;

pub mod api;
pub mod base;
pub mod crypter;
pub mod errors;
pub mod table;
pub mod user;

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
