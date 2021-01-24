use crate::syntax::ast::*;
use crate::syntax::ast::opts::*;

pub mod var;
pub mod typ;

pub use crate::syntax::ast::stat::{
    var::VarStat,
    typ::TypStat };

#[derive(Clone, Debug)]
pub enum Stat {
    illegal,
    r#use,
    def,
    var(VarStat),
    // Fun(fun_stat),
    typ(TypStat),
    ali(TypStat),
    opts(AssOpts),
    ident(String),
    retype(TypOpts),
    r#if,
    when,
    r#loop,
}
