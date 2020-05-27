#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::scanning::token;
use crate::scanning::locate;

use crate::getset;

#[derive(Clone, Debug)]
pub struct ID<T> {
    pub loc: locate::LOCATION,
    pub nod: Box<T>,
}
impl<T> ID<T> {
    pub fn new(loc: locate::LOCATION, nod: T) -> Self { ID{loc, nod: Box::new(nod)} }
    pub fn get(self) -> T { *self.nod }
    pub fn set(&mut self, nod: T) { self.nod = Box::new(nod) }
    pub fn get_loc(self) -> locate::LOCATION { self.loc }
    pub fn set_loc(&mut self, loc: locate::LOCATION) { self.loc = loc }
}

#[derive(Clone, Debug)]
pub enum tree {
    expr(expr),
    stat(stat),
}

pub type expr = ID<expr_type>;
#[derive(Clone, Debug)]
pub enum expr_type {
    Comment,
    Ident,
    Number,
    Letter(letter_expr),
    Container(container_expr),
    Binary(binary_expr),
}

pub type stat = ID<stat_type>;
#[derive(Clone, Debug)]
pub enum stat_type {
    Use,
    Def,
    Var(var_stat),
    Fun(fun_stat),
    Typ(type_expr),
    Ident(String),
    Opts(assign_opts),
    If,
    When,
    Loop,
    Illegal,
}

#[derive(Clone, Debug, GetSet)]
pub struct var_stat{
    options: Vec<stat>,
    ident: stat,
    multi: Option<(usize, String)>,
    retype: Option<stat>,
    body: Option<tree>
}

impl var_stat {
    pub fn init() -> Self {
        var_stat { options: Vec::new(), ident: stat::new(locate::LOCATION::def(), stat_type::Ident(String::new())), multi: None, retype: None, body: None }
    }
}
#[derive(Clone, Debug)]
pub struct fun_stat {
    options: Vec<stat>,
    implement: Option<Box<tree>>,
    ident: Box<String>,
    generics: Option<Box<tree>>,
    parameters: Option<Box<tree>>,
    retype: Option<Box<stat>>,
    body: Box<tree>
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
    elements: Box<tree>
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
