pub mod var;
pub mod typ;
pub mod ali;
pub mod r#use;
pub mod fun;
pub mod lab;
pub use crate::syntax::nodes::stat::assign::{
    ali::NodeStatAssAli,
    typ::NodeStatAssTyp,
    var::NodeStatAssVar,
    lab::NodeStatAssLab,
    r#use::NodeStatAssUse,
    fun::NodeStatAssFun,
};

