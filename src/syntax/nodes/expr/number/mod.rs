use std::fmt;
use crate::syntax::nodes::NodeTrait;

#[derive(Clone)]
pub enum NodeExprNumber {
    int(isize),
    int_8(i8),
}

impl NodeTrait for NodeExprNumber {}

impl fmt::Display for NodeExprNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeExprNumber::int(i) => write!(f, "{}", i),
            NodeExprNumber::int_8(i) => write!(f, "{}", i)
        }
    }
}
