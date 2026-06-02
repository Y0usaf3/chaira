use crate::prelude::*;
use crate::kinds::{AggregateFunction, LinkType, Prefix};
use super::value::Value;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FormulaValue {
    expression: String,
    result: Box<Value>,
}

impl FormulaValue {
    pub fn new(expression: String, value: Value) -> Self {
        FormulaValue {
            expression,
            result: Box::new(value),
        }
    }

    pub fn result(&self) -> &Value {
        &self.result
    }

    pub fn expression(&self) -> &String {
        &self.expression
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RollUpValue {
    link_field_id: FieldId,
    target_field_id: FieldId,
    function: AggregateFunction,
    computed_values: Box<Value>,
}

impl RollUpValue {
    pub fn new(
        link_field_id: FieldId,
        target_field_id: FieldId,
        function: AggregateFunction,
        computed_values: Value,
    ) -> Self {
        Self {
            link_field_id,
            target_field_id,
            function,
            computed_values: Box::new(computed_values),
        }
    }

    pub fn value(&self) -> &Value {
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

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct LookUpValue {
    link_field_id: FieldId,
    target_field_id: FieldId,
    computed_values: Box<Value>,
}

impl LookUpValue {
    pub fn new(link_field_id: FieldId, target_field_id: FieldId, computed_values: Value) -> Self {
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

    pub fn value(&self) -> &Value {
        &self.computed_values
    }
}

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

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct AutoNumberValue {
    value: usize,
    prefix: Prefix,
    formatted: String,
}

impl AutoNumberValue {
    pub fn new(value: usize, prefix: Prefix) -> Self {
        let prefix_str = match prefix {
            Prefix::Dot => '•',
            Prefix::Star => '*',
        };

        let formatted = format!("{}{}", prefix_str, value);
        AutoNumberValue {
            value,
            prefix,
            formatted,
        }
    }

    pub fn formatted(&self) -> &str {
        self.formatted.as_str()
    }

    pub fn prefix(&self) -> &Prefix {
        &self.prefix
    }

    pub fn value(&self) -> &usize {
        &self.value
    }
}
