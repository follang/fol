#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::scan::token;
use crate::scan::locate;

use crate::getset;


#[derive(Clone, Debug)]
pub enum root {
    expr(expr),
    stat(stat),
}

#[derive(Clone, Debug)]
pub struct tree {
    one: root,
    loc: locate::LOCATION,
}
impl tree {
    pub fn new(one: root, loc: locate::LOCATION) -> Self {
        tree{one, loc}
    }
    pub fn node(&self) -> &root {
        &self.one
    }
    pub fn loc(&self) -> &locate::LOCATION {
        &self.loc
    }
}


#[derive(Clone, Debug)]
pub enum expr {
    Comment,
    Ident,
    Number,
    Letter(letter_expr),
    Container(contain_expr),
    Binary(binary_expr)
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug, GetSet)]
pub struct var_stat{
    options: Vec<assign_opts>,
    ident: String,
    retype: Option<expr>,
    body: Option<Box<root>>
}

impl var_stat {
    pub fn init() -> Self {
        var_stat { options: Vec::new(), ident: String::from("test"), retype: None, body: None }
    }
}

#[derive(Clone, Debug)]
pub struct fun_stat {
    options: Vec<assign_opts>,
    implement: Option<Box<root>>,
    ident: String,
    generics: Option<Box<root>>,
    parameters: Option<Box<root>>,
    retype: Option<expr>,
    body: Box<root>
}

#[derive(Clone, Debug)]
pub enum assign_opts {
    Imu, Mut, Sta, Nor, Exp, Hid, Stk, Hep,
}

#[derive(Clone, Debug)]
pub struct contain_expr {
    uniform: bool,
    elements: Box<root>
}

#[derive(Clone, Debug)]
pub enum letter_expr {
    string_n,
    string_r,
    char_n(char),
    char_b(u8)
}

#[derive(Clone, Debug)]
pub enum number_expr {
    int(isize),
    int_8(i8),
}

#[derive(Clone, Debug)]
pub enum binary_expr {
    leaf(number_expr),
    node(Box<binary_expr>, number_expr, Box<binary_expr>)
}


//------------------------------------------------------------------------------------------------------//
//                                          DISPLAY PROPERTIES                                          //
//------------------------------------------------------------------------------------------------------//

impl fmt::Display for root {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            root::expr(_) => { write!(f, "expr") }
            root::stat(stat::Var(var_stat)) => { write!(f, "{}", var_stat) }
            root::stat(stat::Fun(fun_stat)) => { write!(f, "fun") }
            root::stat(_) => { write!(f, "stat") }
        }
    }
}

impl fmt::Display for var_stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "var {:?} {}", self.options, self.ident)
    }
}

impl fmt::Display for assign_opts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            assign_opts::Imu => { write!(f, "imu") }
            assign_opts::Mut => { write!(f, "mut") }
            assign_opts::Sta => { write!(f, "sta") }
            assign_opts::Nor => { write!(f, "nor") }
            assign_opts::Exp => { write!(f, "exp") }
            assign_opts::Hid => { write!(f, "hid") }
            assign_opts::Stk => { write!(f, "stk") }
            assign_opts::Hep => { write!(f, "hep") }
        }
    }
}
