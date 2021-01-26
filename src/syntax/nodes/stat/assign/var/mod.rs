use std::fmt;
use crate::syntax::nodes::{Node, Stat, Opts};

#[derive(Clone)]
pub struct VarStat{
    options: Vec<Box<dyn Opts>>,
    ident: Box<dyn Node>,
    data: Box<dyn Node>,
    body: Option<Box<dyn Node>>,
}

impl Node for VarStat {}
impl Stat for VarStat {}

impl fmt::Display for VarStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
