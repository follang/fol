use dyn_clone::DynClone;
use crate::types::*;

pub mod datatype;
pub mod assign;
pub mod ident;

pub use crate::syntax::nodes::stat::{
        datatype::*,
        assign::*,
        ident::*,
};


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
