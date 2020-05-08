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
    var_{
        options: Vec<token::OPTION>,
        ident: String,
        retype: token::TYPE,
        value: EXPR
    },
    fun_{
        options: Vec<token::OPTION>,
        implement: Vec<token::KEYWORD>,
        ident: String,
        generics: Vec<STAT>,
        parameters: Vec<STAT>,
        retype: token::TYPE,
        value: EXPR
    },
    typ_,
    if_,
    when_,
    loop_,
}

