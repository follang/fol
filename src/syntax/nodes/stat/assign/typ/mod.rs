use std::fmt;
use crate::syntax::nodes::{NodeTrait, StatTrait};

#[derive(Clone)]
pub struct TypStatTrait {
    optis: Vec<Box<dyn NodeTrait>>,
    ident: Box<dyn NodeTrait>,
    generics: Option<Vec<(Box<dyn NodeTrait>, Box<dyn NodeTrait>)>>,
    contract: Option<Vec<Box<dyn NodeTrait>>>,
    form: Option<Box<dyn NodeTrait>>,
    body: Option<Box<dyn NodeTrait>>,
}

impl NodeTrait for TypStatTrait {}
impl StatTrait for TypStatTrait {}

impl fmt::Display for TypStatTrait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
