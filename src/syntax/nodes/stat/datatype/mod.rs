mod types;

use std::fmt;
use crate::syntax::nodes::{NodeTrait, StatTrait};
pub use crate::syntax::nodes::stat::datatype::types::*;

#[derive(Clone)]
pub struct DatStatTrait {
    token: Datatype,
    optis: Vec<Box<dyn NodeTrait>>,
}

impl NodeTrait for DatStatTrait {}
impl StatTrait for DatStatTrait {}

impl fmt::Display for DatStatTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
