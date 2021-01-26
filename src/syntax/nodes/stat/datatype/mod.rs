mod types;

use std::fmt;
use crate::syntax::nodes::{Node, Stat};
pub use crate::syntax::nodes::stat::datatype::types::*;

#[derive(Clone)]
pub struct DatStat {
    token: Datatype,
    optis: Vec<Box<dyn Node>>,
}

impl Node for DatStat {}
impl Stat for DatStat {}

impl fmt::Display for DatStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
