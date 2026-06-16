use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct LookUpValue {
    link_field_id: FieldId,
    target_field_id: FieldId,
    computed_values: Box<super::value::Value>,
}

impl LookUpValue {
    pub fn new(
        link_field_id: FieldId,
        target_field_id: FieldId,
        computed_values: super::value::Value,
    ) -> Self {
        Self {
            link_field_id,
            target_field_id,
            computed_values: Box::new(computed_values),
        }
    }
    pub fn link_field_id(&self) -> &FieldId {
        &self.link_field_id
    }
    pub fn target_field_id(&self) -> &FieldId {
        &self.target_field_id
    }

    pub fn value(&self) -> &super::value::Value {
        &self.computed_values
    }
}
