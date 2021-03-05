use dyn_clone::DynClone;
use crate::types::*;

pub mod datatype;
pub mod ident;
pub mod var;
pub mod typ;
pub use crate::syntax::nodes::stat::{
        typ::NodeStatAssTyp,
        var::NodeStatAssVar,
        datatype::*,
        ident::*,
};
