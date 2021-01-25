use dyn_clone::DynClone;

pub mod var;
pub mod typ;
pub use crate::syntax::nodes::{NodeTrait, id};
pub use crate::syntax::nodes::stat::{
    var::VarStat,
    typ::TypStat };

pub trait StatTrait: NodeTrait {}
dyn_clone::clone_trait_object!(StatTrait);

pub type Stat = id<Box<dyn StatTrait + 'static>>;

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
