#![allow(dead_code)]
#![allow(non_camel_case_types)]

use std::fmt;
use crate::syntax::point;

pub trait Node: core::fmt::Display {}

pub mod expr;
pub mod stat;

#[derive(Clone, Debug)]
pub struct ID<T> {
    pub loc: point::Location,
    pub nod: Box<T>,
}
impl<T> ID<T> {
    pub fn new(loc: point::Location, nod: T) -> Self {
        ID {
            loc,
            nod: Box::new(nod),
        }
    }
    pub fn loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn get(&self) -> &T {
        &self.nod
    }
    pub fn set(&mut self, nod: T) {
        self.nod = Box::new(nod)
    }
}

pub type Tree = ID<tree_type>;
pub type Trees = Vec<Tree>;

#[derive(Clone, Debug)]
pub enum tree_type {
    expr(expr::Expr),
    stat(stat::stat_type),
}

