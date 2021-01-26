use std::fmt;
use crate::syntax::nodes::{NodeTrait, StatTrait, OptsTrait};

#[derive(Clone)]
pub struct VarStatTrait{
    options: Vec<Box<dyn OptsTrait>>,
    ident: Box<dyn NodeTrait>,
    data: Box<dyn NodeTrait>,
    body: Option<Box<dyn NodeTrait>>,
}

impl NodeTrait for VarStatTrait {}
impl StatTrait for VarStatTrait {}

impl fmt::Display for VarStatTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
