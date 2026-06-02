use crate::prelude::*;
use iso_currency::CurrencySymbol;
use ordered_float::OrderedFloat;
use surrealdb_types::Value as XValue;

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct NumberValue {
    value: usize,
}

impl NumberValue {
    pub fn new(value: Option<usize>, default: Option<usize>) -> Result<Self, super::ValueError> {
        if value.is_none() && default.is_none() {
            return Err(super::ValueError::MissingValue);
        };
        if let Some(v) = value {
            Ok(NumberValue { value: v })
        } else {
            Ok(NumberValue {
                value: default.unwrap_or(0),
            })
        }
    }
    pub fn value(&self) -> &usize {
        &self.value
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DecimalValue {
    value: OrderedFloatIThink,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct OrderedFloatIThink(pub OrderedFloat<f64>);

impl surrealdb_types::SurrealValue for OrderedFloatIThink {
    fn kind_of() -> Kind {
        Kind::Float
    }

    fn into_value(self) -> XValue {
        use surrealdb_types::Number;
        XValue::Number(Number::Float(self.0.0))
    }

    fn from_value(value: XValue) -> Result<Self, surrealdb_types::Error> {
        use surrealdb_types::Number;
        match value {
            XValue::Number(Number::Float(n)) => Ok(OrderedFloatIThink(OrderedFloat::<f64>(n))),
            _ => Err(surrealdb_types::Error::thrown(
                "Expected a number for DecimalValue".to_string(),
            )),
        }
    }

    fn is_value(value: &XValue) -> bool {
        matches!(value, XValue::Number(_))
    }
}

impl DecimalValue {
    pub fn new(value: Option<f64>, default: Option<f64>) -> Result<Self, super::ValueError> {
        if value.is_none() && default.is_none() {
            return Err(super::ValueError::MissingValue);
        };
        if let Some(v) = value {
            Ok(DecimalValue {
                value: OrderedFloatIThink(OrderedFloat::from(v)),
            })
        } else {
            Ok(DecimalValue {
                value: OrderedFloatIThink(OrderedFloat::from(default.unwrap_or(0.0))),
            })
        }
    }

    pub fn value(&self) -> &OrderedFloat<f64> {
        &self.value.0
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PercentValue {
    value: i32,
}

impl PercentValue {
    pub fn new(value: i32) -> Self {
        PercentValue { value }
    }
    pub fn value(&self) -> &i32 {
        &self.value
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CurrencyValue {
    value: i64,
    currency_symbole: String,
    formatted: String,
}

impl CurrencyValue {
    pub fn new(value: i64, currency_symbole: CurrencySymbol) -> Self {
        let formatted = format!("{} {}", value, &currency_symbole.symbol);
        CurrencyValue {
            value,
            currency_symbole: currency_symbole.to_string(),
            formatted,
        }
    }

    pub fn value_as_int(&self) -> &i64 {
        &self.value
    }

    pub fn value_as_str(&self) -> &str {
        &self.formatted
    }
}

#[derive(Debug, Clone, SurrealValue, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct RatingValue {
    value: u8,
}

impl RatingValue {
    pub fn new(value: Option<u8>, max: u8) -> Result<Self, super::ValueError> {
        let ratings = value.unwrap_or(0);
        if ratings > max {
            return Err(super::ValueError::RatingExceedsMax {
                value: ratings,
                max,
            });
        };
        Ok(Self { value: ratings })
    }
    pub fn value(&self) -> &u8 {
        &self.value
    }
}
