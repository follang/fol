#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt;
use dyn_clone::DynClone;
use crate::syntax::point;


pub mod expr;
pub mod stat;
pub mod opts;

pub use crate::syntax::nodes::expr::*;
pub use crate::syntax::nodes::stat::*;
pub use crate::syntax::nodes::opts::*;

#[derive(Clone)]
pub struct id<T: ?Sized> {
    pub loc: point::Location,
    pub node: T,
}

impl<T> id<T> {
    pub fn new(loc: point::Location, node: T) -> Self {
        Self{ loc, node }
    }
    pub fn loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn get(&self) -> &T {
        &self.node
    }
    pub fn set(&mut self, node: T) {
        self.node = node
    }
}

pub trait NodeTrait: DynClone + fmt::Display {}
dyn_clone::clone_trait_object!(NodeTrait);

pub type Node = id<Box<dyn NodeTrait + 'static>>;
