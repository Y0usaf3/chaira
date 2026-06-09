use crate::prelude::*;
use ordered_float::OrderedFloat;
use surrealdb::types::Value as XValue;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct OrderedFloatIThink(pub OrderedFloat<f64>);

impl SurrealValue for OrderedFloatIThink {
    fn kind_of() -> Kind {
        Kind::Float
    }

    fn into_value(self) -> XValue {
        use surrealdb::types::Number;
        XValue::Number(Number::Float(self.0.0))
    }

    fn from_value(value: XValue) -> Result<Self, surrealdb::types::Error> {
        use surrealdb::types::Number;
        match value {
            XValue::Number(Number::Float(n)) => Ok(OrderedFloatIThink(OrderedFloat::<f64>(n))),
            _ => Err(surrealdb::types::Error::thrown(
                "Expected a number for DecimalValue".to_string(),
            )),
        }
    }

    fn is_value(value: &XValue) -> bool {
        matches!(value, XValue::Number(_))
    }
}
