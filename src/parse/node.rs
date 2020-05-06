#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
use crate::scan::token;

pub enum NODE {
    expr(EXPR),
    stat(STAT),
    comp(Vec<NODE>)
}

pub enum EXPR {
    ident_,
    literal,
    comment,
    container(Vec<EXPR>)
}

pub enum STAT {
    use_,
    def_,
    var_,
    pro_,
    typ_,
    if_,
    when_,
    loop_,
}

pub struct ROOT {
    el: Vec<NODE>
}
