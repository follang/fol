#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt;
use crate::syntax::point;


pub mod expr;
pub mod stat;
pub mod opts;

#[derive(Clone, Debug)]
pub struct id<T> {
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

pub type Tree = id<tree_type>;
pub type Trees = Vec<Tree>;

pub type Node = id<dyn Ast + 'static>;
pub trait Ast {}

#[derive(Clone, Debug)]
pub enum tree_type {
    expr(expr::Expr),
    stat(stat::Stat),
}

