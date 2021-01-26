#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt;
use dyn_clone::DynClone;
use crate::syntax::point;

pub mod expr;
pub use crate::syntax::nodes::expr::*;
pub mod stat;
pub use crate::syntax::nodes::stat::*;

pub trait Node: DynClone + fmt::Display {}
dyn_clone::clone_trait_object!(Node);
