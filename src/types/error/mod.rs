#![allow(dead_code)]
use dyn_clone::DynClone;

pub mod flaw;
pub mod typo;
pub mod slip;

use std::fmt;
use colored::Colorize;
use crate::syntax::point;
pub use crate::types::error::{flaw::Flaw, typo::Typo, slip::Slip};

pub trait Glitch: std::error::Error + DynClone {}
dyn_clone::clone_trait_object!(Glitch);
pub type Errors = Vec<Box<(dyn Glitch + 'static)>>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Repo { Error }
impl std::error::Error for Repo  {  }
impl Glitch for Repo {  }

impl fmt::Display for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\n{}", " Repo ".black().on_red(),
        )
    }
}
