#![allow(dead_code)]
use dyn_clone::DynClone;
use terminal_size::{Width, Height, terminal_size};

pub mod flaw;
pub mod typo;
pub mod slip;

use std::fmt;
use colored::Colorize;
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

pub fn border_up(chr: &str, msg: String) -> String {
    let mut width = if let Some((Width(w), Height(h))) = terminal_size() { w as usize } else { 5 };
    width = width - msg.len();
    let middle = 5;
    format!("{}{}{}\n", chr.repeat(width - middle).bright_black(), msg.red(), chr.repeat(middle).bright_black())
}

fn border_down(chr: &str, msg: String) -> String {
    let mut width = if let Some((Width(w), Height(h))) = terminal_size() { w as usize } else { 5 };
    width = width - msg.len();
    let middle = 5;
    format!("\n{}{}{}", chr.repeat(width - middle).bright_black(), msg.red(), chr.repeat(middle).bright_black())
}
