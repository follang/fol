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
