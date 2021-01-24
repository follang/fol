#![allow(dead_code)]

pub mod flaw;
pub mod typo;
pub mod slip;

use std::fmt;
use colored::Colorize;
use crate::syntax::point;

pub use crate::types::error::{flaw::Flaw, typo::Typo, slip::Slip};
pub trait Glitch: std::error::Error {}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Fault {
    Flaw(Flaw),
    Typo(Typo),
    Slip(Slip)
}

impl fmt::Display for Fault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Fault::Flaw(v) => write!(f, "{}", v),
            Fault::Typo(v) => write!(f, "{}", v),
            Fault::Slip(v) => write!(f, "{}", v),
        }
    }
}
impl std::error::Error for Fault  {  }
