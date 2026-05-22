//use models::*;
use surrealdb_types::SurrealValue;
use surrealism::surrealism;

#[surrealism]
pub fn can(mask: u32, flag: u32) -> bool {
    (mask & 1) == 1 || (mask & flag) == flag
}
