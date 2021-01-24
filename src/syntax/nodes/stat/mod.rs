use crate::syntax::nodes::*;
use crate::syntax::nodes::opts::*;

pub mod var;
pub mod typ;

pub use crate::syntax::nodes::stat::{
    var::VarStat,
    typ::TypStat };

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
