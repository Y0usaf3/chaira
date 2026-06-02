use crate::prelude::*;
use surrealdb_types::Value as XValue;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AttachmentItem {
    pub(crate) file_id: Uuid,
    pub(crate) name: String,
    pub(crate) mime_type: Meme,
    pub(crate) size: usize,
    pub(crate) uploaded_at: Datetime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Meme(pub String);

impl SurrealValue for Meme {
    fn kind_of() -> Kind {
        Kind::String
    }

    fn into_value(self) -> XValue {
        XValue::String(self.0)
    }

    fn from_value(value: XValue) -> Result<Self, surrealdb_types::Error> {
        match value {
            XValue::String(s) => Ok(Meme(s)),
            _ => Err(surrealdb_types::Error::thrown(
                "Expected a string (Strand) for MimeWrapper".to_string(),
            )),
        }
    }

    fn is_value(value: &XValue) -> bool {
        matches!(value, XValue::String(_))
    }
}

impl AttachmentItem {
    pub fn new(file_id: Uuid, name: String, mime_type: mime::Mime, size: usize) -> Self {
        Self {
            file_id,
            name,
            mime_type: Meme(mime_type.to_string()),
            size,
            uploaded_at: Datetime::now(),
        }
    }

    pub fn mime_str(&self) -> String {
        self.mime_type.0.clone()
    }

    pub fn readability_mime(&self) -> String {
        self.mime_type.0.clone()
    }

    pub fn readable_size(&self) -> String {
        format!("{} Mb", self.size / (1024 * 1024))
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AttachmentValue {
    files: Vec<AttachmentItem>,
}

impl AttachmentValue {
    pub fn new(files: Vec<AttachmentItem>) -> Self {
        Self { files }
    }
    pub fn value(&self) -> Vec<AttachmentItem> {
        self.files.clone()
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct JsonValue {
    pub value: String,
}

impl JsonValue {
    pub fn new(value: String) -> Result<Self, super::ValueError> {
        serde_json::from_str::<serde_json::Value>(&value)
            .map_err(|e| super::ValueError::InvalidJson(e.to_string()))?;
        Ok(JsonValue { value })
    }

    pub fn value(&self) -> String {
        self.value.clone()
    }
}
