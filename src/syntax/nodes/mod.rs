#![allow(non_camel_case_types)]

use std::fmt;
use dyn_clone::DynClone;
use crate::types::*;

// pub mod expr;
// pub use crate::syntax::nodes::expr::*;
pub mod stat;
pub use crate::syntax::nodes::stat::*;

pub trait NodeTrait: DynClone + fmt::Display {}
dyn_clone::clone_trait_object!(NodeTrait);

pub type Node = id::ID<Box<dyn NodeTrait>>;
pub type Nodes = List<Node>;
pub type Pools = Pool<Node>;
