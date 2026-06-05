use crate::prelude::*;
use super::attachment_item::AttachmentItem;

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
