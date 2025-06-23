use std::fmt;
use dyn_clone::DynClone;
use fol_types::*;

// pub mod expr;
// pub use crate::nodes::expr::*;
pub mod stat;
pub use crate::nodes::stat::*;

pub trait NodeTrait: DynClone + fmt::Display {}
dyn_clone::clone_trait_object!(NodeTrait);

pub type Node = id::ID<Box<dyn NodeTrait>>;
pub type Nodes = List<Node>;