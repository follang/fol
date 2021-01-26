use dyn_clone::DynClone;

pub use crate::syntax::nodes::Node;
pub mod datatype;
pub use crate::syntax::nodes::stat::datatype::*;
pub mod assign;
pub use crate::syntax::nodes::stat::assign::*;

pub trait Stat: Node {}
dyn_clone::clone_trait_object!(Stat);

pub trait Opts: Node {}
dyn_clone::clone_trait_object!(Opts);



// STATEMENTS TYPES
// - illegal,
// - r#use,
// - def,
// - var(VarStat),
// - fun(FunStat),
// - typ(TypStat),
// - ali(TypStat),
// - opts(AssOpts),
// - ident(String),
// - retype(TypOpts),
// - if,
// - when,
// - loop,
