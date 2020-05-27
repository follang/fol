#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::parsing::ast::*;

impl fmt::Display for body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            body::expr(_) => { write!(f, "expr") }
            body::stat(stat::Var(var_stat)) => { write!(f, "{}", var_stat) }
            body::stat(stat::Fun(fun_stat)) => { write!(f, "fun") }
            body::stat(_) => { write!(f, "stat") }
        }
    }
}


impl fmt::Display for stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            stat::Typ(a) => {write!(f, "{}", a)},
            _ => { write!(f, "---") }
        }
    }
}


impl fmt::Display for var_stat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut typ = String::new();
        let mut base = String::new();
        if let Some(a) = self.get_retype().clone() {
            typ = ": ".to_string() + a.to_string().as_str() + "[]";
        }
        if let Some(a) = self.get_multi().clone() {
            base = "[".to_string() + a.0.to_string().as_str() + " + " + a.1.as_str() + "]";
        }
        // let typ = match self.get_retype().clone() {
            // Some(a) => { a.to_string() },
            // None => { "NONE".to_string() }
        // };
        write!(f, "{:<15}var{:?} {}{};", base, self.get_options(), self.get_ident(), typ)
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

impl fmt::Display for type_expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            type_expr::Int => { write!(f, "int") },
            type_expr::Flt => { write!(f, "flt") },
            type_expr::Chr => { write!(f, "chr") },
            type_expr::Bol => { write!(f, "bol") },
            type_expr::Arr => { write!(f, "arr") },
            type_expr::Vec => { write!(f, "vec") },
            type_expr::Seq => { write!(f, "seq") },
            type_expr::Mat => { write!(f, "mat") },
            type_expr::Set => { write!(f, "set") },
            type_expr::Map => { write!(f, "map") },
            type_expr::Axi => { write!(f, "axi") },
            type_expr::Tab => { write!(f, "tab") },
            type_expr::Str => { write!(f, "str") },
            type_expr::Num => { write!(f, "num") },
            type_expr::Ptr => { write!(f, "ptr") },
            type_expr::Err => { write!(f, "err") },
            type_expr::Opt => { write!(f, "opt") },
            type_expr::Nev => { write!(f, "nev") },
            type_expr::Uni => { write!(f, "uni") },
            type_expr::Any => { write!(f, "any") },
            type_expr::Non => { write!(f, "non") },
            type_expr::Nil => { write!(f, "nil") },
            type_expr::Rec => { write!(f, "rec") },
            type_expr::Ent => { write!(f, "ent") },
            type_expr::Blu => { write!(f, "blu") },
            type_expr::Std => { write!(f, "std") },
            type_expr::Loc => { write!(f, "loc") },
            type_expr::Url => { write!(f, "url") },
            type_expr::Blk => { write!(f, "blk") },
            type_expr::Rut => { write!(f, "rut") },
            type_expr::Pat => { write!(f, "pat") },
            type_expr::Gen => { write!(f, "gen") },
        }
    }
}
