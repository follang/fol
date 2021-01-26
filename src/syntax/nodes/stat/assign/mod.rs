pub mod opts;
pub mod var;
pub mod typ;
pub use crate::syntax::nodes::stat::assign::{
    var::VarStatTrait,
    typ::TypStatTrait };

