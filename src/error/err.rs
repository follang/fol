#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


extern crate colored;
use colored::Colorize;

use std::fmt;
use crate::scan::locate;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum flaw_type {
    lexer,
    parser,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct flaw {
    typ: flaw_type,
    msg: String,
    loc: locate::LOCATION,
}

impl flaw {
    pub fn new(typ: flaw_type, msg: &str, loc: locate::LOCATION) -> Self {
        flaw { typ, msg: msg.to_string(), loc }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct FLAW {
    el: Vec<flaw>
}

impl FLAW {
    pub fn init() -> Self {
        FLAW{ el: Vec::new() }
    }
    pub fn list(&self) -> &Vec<flaw> {
        &self.el
    }
    pub fn report(&mut self, typ: flaw_type, msg: &str, loc:locate::LOCATION) {
        let e = flaw::new(typ, msg, loc);
        &self.el.push(e);
    }
    pub fn show(&mut self) {
        for e in self.el.iter() {
            println!("{}", e);
        }
    }
}

fn get_line_at(filepath: &str, line_num: usize) -> String {
    let file = File::open(Path::new(filepath)).unwrap();
    let mut lines = BufReader::new(&file).lines();
    lines.nth(line_num-1).unwrap().unwrap()
}

impl fmt::Display for flaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let errtype: &str = match self.typ {
            flaw_type::lexer => " flaw in lexing stage ",
            flaw_type::parser => " flaw in parsing stage ",
        };
        write!(f,
            "\n\n{}\n {}\n\n    {}\n    {}{}\n {}",
            errtype.black().bold().on_white(),
            self.loc,
            get_line_at(self.loc.path(), self.loc.row()).red(),
            " ".repeat(self.loc.col()-1),
            "^".repeat(self.loc.len()),
            self.msg,
        )
    }
}
