#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::scan::token;
use crate::scan::locate;

use crate::getset;

#[derive(Clone, Debug)]
pub struct Located<T> {
    pub location: locate::LOCATION,
    pub node: T,
}

#[derive(Clone, Debug)]
pub enum body {
    expr(expr),
    stat(stat),
}

#[derive(Clone, Debug)]
pub struct tree {
    one: body,
    loc: locate::LOCATION,
}
impl tree {
    pub fn new(one: body, loc: locate::LOCATION) -> Self {
        tree{one, loc}
    }
    pub fn node(&self) -> &body {
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
    Container(container_expr),
    Binary(binary_expr),
}

#[derive(Clone, Debug)]
pub enum stat {
    Use,
    Def,
    Var(var_stat),
    Fun(fun_stat),
    Type(type_expr),
    If,
    When,
    Loop,
}

#[derive(Clone, Debug, GetSet)]
pub struct var_stat{
    options: Vec<assign_opts>,
    ident: Box<String>,
    multi: Option<(usize, String)>,
    retype: Option<Box<stat>>,
    body: Option<Box<body>>
}

impl var_stat {
    pub fn init() -> Self {
        var_stat { options: Vec::new(), ident: Box::new(String::new()), multi: None, retype: None, body: None }
    }
}

#[derive(Clone, Debug)]
pub struct fun_stat {
    options: Vec<assign_opts>,
    implement: Option<Box<body>>,
    ident: Box<String>,
    generics: Option<Box<body>>,
    parameters: Option<Box<body>>,
    retype: Option<Box<stat>>,
    body: Box<body>
}

#[derive(Clone, Debug)]
pub enum assign_opts {
    Imu, Mut, Sta, Nor, Exp, Hid, Stk, Hep,
}

#[derive(Clone, Debug)]
pub enum type_expr {
    Int,
    Flt,
    Chr,
    Bol,
    Arr,
    Vec,
    Seq,
    Mat,
    Set,
    Map,
    Axi,
    Tab,
    Str,
    Num,
    Ptr,
    Err,
    Opt,
    Nev,
    Uni,
    Any,
    Non,
    Nil,
    Rec,
    Ent,
    Blu,
    Std,
    Loc,
    Url,
    Blk,
    Rut,
    Pat,
    Gen,
}

#[derive(Clone, Debug)]
pub struct container_expr {
    uniform: bool,
    elements: Box<body>
}

#[derive(Clone, Debug)]
pub enum letter_expr {
    string_normal,
    string_raw,
    string_formated,
    char_normal(char),
    char_binary(u8)
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
