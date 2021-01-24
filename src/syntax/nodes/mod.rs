#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt;
use crate::syntax::point;


pub mod expr;
pub mod stat;
pub mod opts;

pub use crate::syntax::nodes::stat::*;
pub use crate::syntax::nodes::expr::*;
pub use crate::syntax::nodes::opts::*;

pub struct id<T: ?Sized> {
    pub loc: point::Location,
    pub node: Box<T>,
}

impl<T> id<T> {
    pub fn new(loc: point::Location, node: T) -> Self {
        Self{ loc, node: Box::new(node) }
    }
    pub fn loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn get(&self) -> &T {
        &self.node
    }
    pub fn set(&mut self, node: T) {
        self.node = Box::new(node)
    }
}

pub trait Tree {}
pub type Node = id<dyn Tree + 'static>;
