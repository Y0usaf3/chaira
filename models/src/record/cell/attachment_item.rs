use super::meme::Meme;
use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct AttachmentItem {
    pub(crate) file_id: Uuid,
    pub(crate) name: String,
    pub(crate) mime_type: Meme,
    pub(crate) size: usize,
    pub(crate) uploaded_at: Datetime,
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
