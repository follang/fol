use dyn_clone::DynClone;

pub use crate::syntax::nodes::{NodeTrait, Node, id};
pub mod datatype;
pub use crate::syntax::nodes::stat::datatype::*;
pub mod assign;
pub use crate::syntax::nodes::stat::assign::*;

pub trait StatTrait: NodeTrait {}
dyn_clone::clone_trait_object!(StatTrait);
impl NodeTrait for Box<dyn StatTrait> {}

pub type Stat = id<Box<dyn StatTrait>>;
impl From<Stat> for Node {
    fn from(stat: Stat) -> Self {
        Self {
            loc: stat.get_loc().clone(), 
            node: Box::new(stat.get_node().clone())
        }
    }
}

pub trait OptsTrait: NodeTrait {}
dyn_clone::clone_trait_object!(OptsTrait);
impl NodeTrait for Box<dyn OptsTrait> {}

pub type Opts = id<Box<dyn OptsTrait>>;
impl From<Opts> for Node {
    fn from(opts: Opts) -> Self {
        Self {
            loc: opts.get_loc().clone(), 
            node: Box::new(opts.get_node().clone())
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
