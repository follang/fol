use dyn_clone::DynClone;
use crate::types::*;

pub mod datatype;
pub mod ident;
pub mod decshort;
pub mod declong;
pub use crate::syntax::nodes::stat::{
        declong::NodeStatAssTyp,
        decshort::NodeStatAssVar,
        datatype::*,
        ident::*,
};
