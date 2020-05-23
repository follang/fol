#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::scan::token;
use crate::scan::locate;

use crate::getset;

#[derive(Clone, Debug)]
pub struct ID<T> {
    pub loc: locate::LOCATION,
    pub nod: T,
}
impl<T> ID<T> {
    pub fn new(loc: locate::LOCATION, nod: T) -> Self { ID{loc, nod} }
    pub fn get_loc(self) -> locate::LOCATION { self.loc }
    pub fn set_loc(&mut self, loc: locate::LOCATION) { self.loc = loc }
    pub fn get_nod(self) -> T { self.nod }
    pub fn set_nod(&mut self, nod: T) { self.nod = nod }
}

#[derive(Clone, Debug)]
pub enum body {
    expr(expr),
    stat(stat),
}

pub type tree = ID<body>;


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
