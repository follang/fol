pub mod opts;
pub mod var;
pub mod typ;
pub use crate::syntax::nodes::stat::assign::{
    typ::NodeStatAssTyp,
    var::NodeStatAssVar,
};

