use dyn_clone::DynClone;
use crate::types::*;

pub mod container;
pub mod letter;
pub mod number;
pub mod binary;
pub use crate::syntax::nodes::expr::{
    letter::NodeExprLetter,
    container::NodeExprContainer,
    binary::NodeExprBinary,
    number::NodeExprNumber 
};
