pub mod opts;
pub mod var;
pub mod typ;
pub mod ali;
pub mod r#use;
pub mod fun;
pub use crate::syntax::nodes::stat::assign::{
    ali::NodeStatAssAli,
    typ::NodeStatAssTyp,
    var::NodeStatAssVar,
    r#use::NodeStatAssUse,
    fun::NodeStatAssFun,
};

