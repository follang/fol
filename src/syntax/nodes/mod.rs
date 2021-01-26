#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt;
use dyn_clone::DynClone;
use crate::syntax::point;

pub mod expr;
pub use crate::syntax::nodes::expr::*;
pub mod stat;
pub use crate::syntax::nodes::stat::*;

pub struct id<T: ?Sized + fmt::Display> {
    pub loc: point::Location,
    pub node: T,
}

impl<T: std::fmt::Display> id<T> {
    pub fn new(loc: point::Location, node: T) -> Self {
        Self{ loc, node: node }
    }
    pub fn get_loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn get_node(&self) -> &T {
        &self.node
    }
    pub fn set_node(&mut self, node: T) {
        self.node = node
    }
}

pub type Node = id<Box<dyn NodeTrait>>;
pub trait NodeTrait: DynClone + fmt::Display {}
dyn_clone::clone_trait_object!(NodeTrait);
