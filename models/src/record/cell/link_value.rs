use crate::prelude::*;
use crate::kinds::LinkType;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LinkValue {
    pub target_table_id: TableId,
    pub record_ids: Vec<RecordId>,
    pub link_type: LinkType,
}

impl LinkValue {
    pub fn new(target_table_id: TableId, link_type: LinkType, record_ids: Vec<RecordId>) -> Self {
        let final_ids = if link_type == LinkType::OneToOne && record_ids.len() > 1 {
            vec![record_ids[0].clone()]
        } else {
            record_ids
        };

        LinkValue {
            target_table_id,
            link_type,
            record_ids: final_ids,
        }
    }

    pub fn record_ids(&self) -> &[RecordId] {
        &self.record_ids
    }
}
