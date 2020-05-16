#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
use crate::scan::token;
use crate::scan::locate;

#[derive(Clone, Debug)]
pub enum node {
    expr(expr),
    stat(stat),
    node(Vec<node>),
}

#[derive(Clone, Debug)]
pub struct tree {
    node: node,
    loc: locate::LOCATION,
}
impl tree {
    pub fn new(node: node, loc: locate::LOCATION) -> Self {
        tree{node, loc}
    }
    pub fn node(&self) -> &node {
        &self.node
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

#[derive(Clone, Debug)]
pub struct var_stat{
    pub options: Vec<assign_opts>,
    pub ident: String,
    pub retype: Option<expr>,
    pub body: Option<expr>
}

impl var_stat {
    pub fn init() -> Self {
        var_stat { options: Vec::new(), ident: String::from("test"), retype: None, body: None }
    }
    pub fn new(options: Vec<assign_opts>, ident: String, retype: Option<expr>, body: Option<expr>) -> Self {
        var_stat { options, ident, retype, body }
    }
}

#[derive(Clone, Debug)]
pub struct fun_stat {
    options: Vec<assign_opts>,
    implement: Option<Box<node>>,
    ident: String,
    generics: Option<Box<node>>,
    parameters: Option<Box<node>>,
    retype: Option<expr>,
    body: expr
}

#[derive(Clone, Debug)]
pub enum assign_mut {
    Imu,
    Mut,
    Sta,
}

#[derive(Clone, Debug)]
pub enum assign_exp {
    Nor,
    Exp,
    Hid,
}

#[derive(Clone, Debug)]
pub enum assign_ptr {
    Stk,
    Hep,
}

#[derive(Clone, Debug)]
pub enum assign_opts {
    Mut(assign_mut),
    Exp(assign_exp),
    Ptr(assign_ptr),
}

#[derive(Clone, Debug)]
pub struct contain_expr {
    uniform: bool,
    elements: Box<node>
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

impl fmt::Display for node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            node::expr(_) => { write!(f, "expr") }
            node::stat(stat::Var(var_stat)) => { write!(f, "{}", var_stat) }
            node::stat(stat::Fun(fun_stat)) => { write!(f, "fun") }
            node::stat(_) => { write!(f, "stat") }
            _ => { write!(f, "---") }
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
            assign_opts::Mut(assign_mut::Imu) => { write!(f, "imu") }
            assign_opts::Mut(assign_mut::Mut) => { write!(f, "mut") }
            assign_opts::Mut(assign_mut::Sta) => { write!(f, "sta") }
            assign_opts::Exp(assign_exp::Nor) => { write!(f, "nor") }
            assign_opts::Exp(assign_exp::Exp) => { write!(f, "exp") }
            assign_opts::Exp(assign_exp::Hid) => { write!(f, "hid") }
            assign_opts::Ptr(assign_ptr::Stk) => { write!(f, "stk") }
            assign_opts::Ptr(assign_ptr::Hep) => { write!(f, "hep") }
        }
    }
}
