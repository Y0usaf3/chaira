use crate::prelude::*;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[surreal(crate = "::surrealdb_types")]
pub struct FormulaValue {
    expression: String,
    result: Box<super::value::Value>,
}

impl FormulaValue {
    pub fn new(expression: String, value: super::value::Value) -> Self {
        FormulaValue {
            expression,
            result: Box::new(value),
        }
    }

    pub fn result(&self) -> &super::value::Value {
        &self.result
    }

    pub fn expression(&self) -> &String {
        &self.expression
    }
}
