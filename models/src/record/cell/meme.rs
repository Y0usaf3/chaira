use crate::prelude::*;
use surrealdb::types::Value as XValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Meme(pub String);

impl SurrealValue for Meme {
    fn kind_of() -> Kind {
        Kind::String
    }

    fn into_value(self) -> XValue {
        XValue::String(self.0)
    }

    fn from_value(value: XValue) -> Result<Self, surrealdb::types::Error> {
        match value {
            XValue::String(s) => Ok(Meme(s)),
            _ => Err(surrealdb::types::Error::thrown(
                "Expected a string (Strand) for MimeWrapper".to_string(),
            )),
        }
    }

    fn is_value(value: &XValue) -> bool {
        matches!(value, XValue::String(_))
    }
}
