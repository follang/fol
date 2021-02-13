use dyn_clone::DynClone;
use crate::types::*;

pub use crate::syntax::nodes::{NodeTrait, Node};
pub mod contracts;
pub mod datatype;
pub mod assign;
pub mod ident;

pub use crate::syntax::nodes::stat::{
        contracts::*,
        datatype::*,
        assign::*,
        ident::*,
};


pub trait StatTrait: NodeTrait {}
dyn_clone::clone_trait_object!(StatTrait);
impl NodeTrait for Box<dyn StatTrait> {}
pub type Stat = ID<Box<dyn StatTrait>>;
impl From<Stat> for Node {
    fn from(stat: Stat) -> Self {
        Self {
            key: stat.key().clone(), 
            loc: stat.loc().clone(), 
            node: Box::new(stat.node().clone())
        }
    }
}

// STATEMENTS TYPES
// - illegal,
// - r#use,
// - def,
// - var(VarStatTrait),
// - fun(FunStatTrait),
// - typ(TypStatTrait),
// - ali(TypStatTrait),
// - opts(AssOptsTrait),
// - ident(String),
// - retype(TypOptsTrait),
// - if,
// - when,
// - loop,
