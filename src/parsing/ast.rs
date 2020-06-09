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
    pub fn loc(&self) -> &locate::LOCATION { &self.loc }
    pub fn get(&self) -> &T { &self.nod }
    pub fn set(&mut self, nod: T) { self.nod = Box::new(nod) }
}

pub type tree = ID<tree_type>;
pub type trees = Vec<tree>;
#[derive(Clone, Debug)]
pub enum tree_type {
    expr(expr_type),
    stat(stat_type),
}

#[derive(Clone, Debug)]
pub enum expr_type {
    Illegal,
    Comment,
    Number,
    Letter(letter_expr),
    Container(container_expr),
    Binary(binary_expr),
}

#[derive(Clone, Debug)]
pub enum stat_type {
    Illegal,
    Use,
    Def,
    Var(var_stat),
    // Fun(fun_stat),
    Typ(typ_stat),
    Opts(assign_opts),
    Ident(String),
    Retype(retype_stat),
    If,
    When,
    Loop,
}

#[derive(Clone, Debug, GetSet)]
pub struct var_stat{
    options: Option<trees>,
    multi: Option<(usize, String)>,
    ident: tree,
    retype: Option<tree>,
    body: Option<tree>
}

impl var_stat {
    pub fn init() -> Self {
        var_stat {
        options: None,
        ident: tree::new(locate::LOCATION::def(), tree_type::stat(stat_type::Ident(String::new()))),
        multi: None,
        retype: None,
        body: None }
    }
}

#[derive(Clone, Debug, GetSet)]
pub struct typ_stat{
    options: Option<trees>,
    multi: Option<(usize, String)>,
    ident: tree,
    generics: Option<Vec<(tree, tree)>>,
    contract: Option<Vec<tree>>,
    retype: Option<tree>,
    body: Option<tree>
}
impl typ_stat {
    pub fn init() -> Self {
        typ_stat {
        options: None,
        multi: None,
        ident: tree::new(locate::LOCATION::def(), tree_type::stat(stat_type::Ident(String::new()))),
        generics: None,
        contract: None,
        retype: None,
        body: None }
    }
}


#[derive(Clone, Debug)]
pub enum assign_opts {
    Imu, Mut, Sta, Nor, Exp, Hid, Stk, Hep,
}

#[derive(Clone, Debug)]
pub enum retype_stat {
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
