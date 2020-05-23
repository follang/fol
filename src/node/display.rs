#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]


use std::fmt;
// use getset::{CopyGetters, Getters, MutGetters, Setters};

use crate::node::ast::*;

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
            stat::Type(a) => {write!(f, "{}", a)},
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
            type_expr::Rut => { write!(f, "rut") },
            type_expr::Str => { write!(f, "str") },
            _ => { write!(f, "ANY") }
        }
    }
}
