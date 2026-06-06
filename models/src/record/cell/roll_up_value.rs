use crate::kinds::AggregateFunction;
use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RollUpValue {
    link_field_id: FieldId,
    target_field_id: FieldId,
    function: AggregateFunction,
    computed_values: Box<super::value::Value>,
}

impl RollUpValue {
    pub fn new(
        link_field_id: FieldId,
        target_field_id: FieldId,
        function: AggregateFunction,
        computed_values: super::value::Value,
    ) -> Self {
        Self {
            link_field_id,
            target_field_id,
            function,
            computed_values: Box::new(computed_values),
        }
    }

    pub fn value(&self) -> &super::value::Value {
        &self.computed_values
    }

    pub fn function(&self) -> &AggregateFunction {
        &self.function
    }
    pub fn target_field_id(&self) -> &FieldId {
        &self.target_field_id
    }
    pub fn link_field_id(&self) -> &FieldId {
        &self.link_field_id
    }
}
