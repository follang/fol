#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
use crate::scan::token;
use crate::scan::locate;

pub enum node {
    expr(expr),
    stat(stat),
    node(Vec<node>),
}
pub type tree = (node, locate::LOCATION);


pub enum expr {
    Comment,
    Ident,
    Number,
    Letter(letter_expr),
    Container(contain_expr)
}

pub enum stat {
    Use,
    Def,
    Var(var_stat),
    Fun(fun_stat),
    Typ,
    If,
    When,
    Loop,
}
pub struct var_stat{
    options: (assign_mut, assign_exp),
    ident: String,
    retype: Option<expr>,
    body: Option<expr>
}
impl var_stat {
    pub fn init() -> Self {
        var_stat { options: (assign_mut::Imu, assign_exp::Nor), ident: String::new(), retype: None, body: None }
    }
    pub fn new(options: (assign_mut, assign_exp), ident: String, retype: Option<expr>, body: Option<expr>) -> Self {
        var_stat { options, ident, retype, body }
    }
}
impl fmt::Display for var_stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "var")
    }
}

pub struct fun_stat {
    options: (assign_mut, assign_exp),
    implement: Option<Box<node>>,
    ident: String,
    generics: Option<Box<node>>,
    parameters: Option<Box<node>>,
    retype: Option<expr>,
    body: expr
}


pub enum assign_mut {
    Mut,
    Imu,
    Sta,
}

pub enum assign_exp {
    Exp,
    Nor,
    Hid,
}

pub struct contain_expr {
    uniform: bool,
    elements: Box<node>
}

pub enum letter_expr {
    string_n,
    string_r,
    char_n(char),
    char_b(u8)
}

pub enum number_expr {
    int(isize),
    int_8(i8),
}

impl fmt::Display for node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            node::expr(_) => { write!(f, "expr") }
            node::stat(stat::Var(var_stat)) => { write!(f, "var") }
            node::stat(stat::Fun(fun_stat)) => { write!(f, "fun") }
            node::stat(_) => { write!(f, "stat") }
            _ => { write!(f, "---") }
        }
    }
}
