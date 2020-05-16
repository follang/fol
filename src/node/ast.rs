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
    options: options_men,
    ident: String,
    retype: Option<expr>,
    body: Option<expr>
}
impl var_stat {
    pub fn init() -> Self {
        var_stat { options: options_men(assign_mut::Imu, assign_exp::Nor, assign_ptr::Stk), ident: String::new(), retype: None, body: None }
    }
    pub fn new(options: options_men, ident: String, retype: Option<expr>, body: Option<expr>) -> Self {
        var_stat { options, ident, retype, body }
    }
}

pub struct fun_stat {
    options: options_men,
    implement: Option<Box<node>>,
    ident: String,
    generics: Option<Box<node>>,
    parameters: Option<Box<node>>,
    retype: Option<expr>,
    body: expr
}


pub enum assign_mut {
    Imu,
    Mut,
    Sta,
}

pub enum assign_exp {
    Nor,
    Exp,
    Hid,
}

pub enum assign_ptr {
    Stk,
    Hep,
}

pub struct options_men(assign_mut, assign_exp, assign_ptr);

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


//                      //
//  DISPLAY PROPERTIES  //
//                      //

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
        write!(f, "var {}", self.options)
    }
}

impl fmt::Display for assign_mut {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            assign_mut::Imu => { write!(f, "imu") }
            assign_mut::Mut => { write!(f, "mut") }
            assign_mut::Sta => { write!(f, "sta") }
        }
    }
}

impl fmt::Display for assign_exp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            assign_exp::Nor => { write!(f, "nor") }
            assign_exp::Exp => { write!(f, "exp") }
            assign_exp::Hid => { write!(f, "hid") }
        }
    }
}

impl fmt::Display for assign_ptr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            assign_ptr::Stk => { write!(f, "stk") }
            assign_ptr::Hep => { write!(f, "hep") }
        }
    }
}

impl fmt::Display for options_men {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}, {}, {}]", self.0, self.1, self.2)
    }
}
