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
            tree_type::expr(expr) => { write!(f, "{}", expr.to_string()) }
            tree_type::stat(stat) => { write!(f, "{}", stat.to_string()) }
        }
    }
}

impl fmt::Display for stat_type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            stat_type::Typ(a) => {write!(f, "{}", a)},
            stat_type::Var(a) => {write!(f, "{}", a)},
            stat_type::Opts(a) => {write!(f, "{}", a)},
            stat_type::Ident(a) => {write!(f, "{}", a)},
            stat_type::Retype(a) => {write!(f, "{}", a)},
            stat_type::Illegal => {write!(f, "<unknown>")},
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
        if let Some(a) = self.get_options().clone() { for e in a.iter() { opts.push(e.clone().to_string()) } }
        let id: String = self.get_ident().clone().get().to_string();
        let mut body = String::new();
        if let Some(a) = self.get_body().clone() { body = " = ".to_string() + a.to_string().as_str(); }
        write!(f, "{:<15}var{:?} {}{}{};", base, opts, id, typ, body)
    }
}

impl fmt::Display for typ_stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut typ = String::new();
        if let Some(a) = self.get_retype().clone() { typ = ": ".to_string() + a.to_string().as_str() + "[]"; }
        let mut base = String::new();
        if let Some(a) = self.get_multi().clone() { base = "[".to_string() + a.0.to_string().as_str() + ", " + a.1.as_str() + "]"; }
        let mut opts: Vec<String> = Vec::new();
        if let Some(a) = self.get_options().clone() { for e in a.iter() { opts.push(e.clone().to_string()) } }
        let id: String = self.get_ident().clone().get().to_string();
        let mut body = String::new();
        if let Some(a) = self.get_body().clone() { body = " = ".to_string() + a.to_string().as_str(); }
        write!(f, "{:<15}typ{:?} {}{}{};", base, opts, id, typ, body)
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

impl fmt::Display for retype_stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            retype_stat::Int => { write!(f, "int") },
            retype_stat::Flt => { write!(f, "flt") },
            retype_stat::Chr => { write!(f, "chr") },
            retype_stat::Bol => { write!(f, "bol") },
            retype_stat::Arr => { write!(f, "arr") },
            retype_stat::Vec => { write!(f, "vec") },
            retype_stat::Seq => { write!(f, "seq") },
            retype_stat::Mat => { write!(f, "mat") },
            retype_stat::Set => { write!(f, "set") },
            retype_stat::Map => { write!(f, "map") },
            retype_stat::Axi => { write!(f, "axi") },
            retype_stat::Tab => { write!(f, "tab") },
            retype_stat::Str => { write!(f, "str") },
            retype_stat::Num => { write!(f, "num") },
            retype_stat::Ptr => { write!(f, "ptr") },
            retype_stat::Err => { write!(f, "err") },
            retype_stat::Opt => { write!(f, "opt") },
            retype_stat::Nev => { write!(f, "nev") },
            retype_stat::Uni => { write!(f, "uni") },
            retype_stat::Any => { write!(f, "any") },
            retype_stat::Non => { write!(f, "non") },
            retype_stat::Nil => { write!(f, "nil") },
            retype_stat::Rec => { write!(f, "rec") },
            retype_stat::Ent => { write!(f, "ent") },
            retype_stat::Blu => { write!(f, "blu") },
            retype_stat::Std => { write!(f, "std") },
            retype_stat::Loc => { write!(f, "loc") },
            retype_stat::Url => { write!(f, "url") },
            retype_stat::Blk => { write!(f, "blk") },
            retype_stat::Rut => { write!(f, "rut") },
            retype_stat::Pat => { write!(f, "pat") },
            retype_stat::Gen => { write!(f, "gen") },
        }
    }
}
