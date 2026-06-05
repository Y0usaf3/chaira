use crate::prelude::*;

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
