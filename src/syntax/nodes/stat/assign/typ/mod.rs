use std::fmt;
use crate::syntax::nodes::{Node, Stat};

#[derive(Clone)]
pub struct TypStat {
    optis: Vec<Box<dyn Node>>,
    ident: Box<dyn Node>,
    generics: Option<Vec<(Box<dyn Node>, Box<dyn Node>)>>,
    contract: Option<Vec<Box<dyn Node>>>,
    form: Option<Box<dyn Node>>,
    body: Option<Box<dyn Node>>,
}

impl Node for TypStat {}
impl Stat for TypStat {}

impl fmt::Display for TypStat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!();
    }
}
