#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::parsing::ast::*;

impl fmt::Display for tree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.clone().get() {
            tree_type::expr(expr) => { write!(f, "{}", expr) }
            tree_type::stat(stat) => { write!(f, "{}", stat) }
        }
    }
}

impl fmt::Display for tree_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            tree_type::expr(expr) => { write!(f, "{}", expr.clone().to_string()) }
            tree_type::stat(stat) => { write!(f, "{}", stat.clone().to_string()) }
        }
    }
}

impl fmt::Display for stat_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            stat_type::Typ(a) => {write!(f, "{}", a)},
            stat_type::Var(a) => {write!(f, "{}", a)},
            stat_type::Ident(a) => {write!(f, "{}", a)},
            _ => { write!(f, "---") }
        }
    }
}

impl fmt::Display for expr_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            _ => { write!(f, "---") }
        }
    }
}

impl fmt::Display for var_stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut typ = String::new();
        if let Some(a) = self.get_retype().clone() { typ = ": ".to_string() + a.to_string().as_str() + "[]"; }
        let mut base = String::new();
        if let Some(a) = self.get_multi().clone() { base = "[".to_string() + a.0.to_string().as_str() + ", " + a.1.as_str() + "]"; }
        let mut opts: Vec<String> = Vec::new();
        for e in self.get_options().iter() { opts.push(e.clone().to_string()) }
        let id: String = self.get_ident().clone().get().to_string();
        write!(f, "{:<15}var{:?} {}{};", base, opts, id, typ)
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

impl fmt::Display for typ_expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            typ_expr::Int => { write!(f, "int") },
            typ_expr::Flt => { write!(f, "flt") },
            typ_expr::Chr => { write!(f, "chr") },
            typ_expr::Bol => { write!(f, "bol") },
            typ_expr::Arr => { write!(f, "arr") },
            typ_expr::Vec => { write!(f, "vec") },
            typ_expr::Seq => { write!(f, "seq") },
            typ_expr::Mat => { write!(f, "mat") },
            typ_expr::Set => { write!(f, "set") },
            typ_expr::Map => { write!(f, "map") },
            typ_expr::Axi => { write!(f, "axi") },
            typ_expr::Tab => { write!(f, "tab") },
            typ_expr::Str => { write!(f, "str") },
            typ_expr::Num => { write!(f, "num") },
            typ_expr::Ptr => { write!(f, "ptr") },
            typ_expr::Err => { write!(f, "err") },
            typ_expr::Opt => { write!(f, "opt") },
            typ_expr::Nev => { write!(f, "nev") },
            typ_expr::Uni => { write!(f, "uni") },
            typ_expr::Any => { write!(f, "any") },
            typ_expr::Non => { write!(f, "non") },
            typ_expr::Nil => { write!(f, "nil") },
            typ_expr::Rec => { write!(f, "rec") },
            typ_expr::Ent => { write!(f, "ent") },
            typ_expr::Blu => { write!(f, "blu") },
            typ_expr::Std => { write!(f, "std") },
            typ_expr::Loc => { write!(f, "loc") },
            typ_expr::Url => { write!(f, "url") },
            typ_expr::Blk => { write!(f, "blk") },
            typ_expr::Rut => { write!(f, "rut") },
            typ_expr::Pat => { write!(f, "pat") },
            typ_expr::Gen => { write!(f, "gen") },
        }
    }
}
