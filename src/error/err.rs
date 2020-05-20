#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


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
    parser_unexpected,
    parser_missmatch,
    parser_indentation,
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
        for (i, e) in self.el.iter().enumerate() {
            println!("\n\n{}{:<2} : {}", " ERROR #".black().on_red(), (&i + 1).to_string().black().on_red(), e);
        }
        if self.el.len() != 0 {
            let num = if self.el.len() == 1 { "error" } else { "errors" };
            println!("\n\n{:^10} due to {:^3} previous {}", "ABORTING".black().on_red(), self.el.len().to_string().black().on_red(), num);
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
        let parse_msg = String::from(" flaw in parsing stage ").bold().on_white();
        let lexer_msg = String::from(" flaw in lexing stage ").bold().on_white();
        let separator = "".white().to_string();
        let errtype = match self.typ {
            flaw_type::lexer => lexer_msg.to_string(),
            flaw_type::parser_unexpected =>{ parse_msg.to_string() + separator.as_str() +  &" UNEXPECTED TOKEN ".bold().on_red().to_string() },
            flaw_type::parser_missmatch => { parse_msg.to_string() + separator.as_str() + &" MISSMATCHED ARGUMENTS ".bold().on_red().to_string() },
            flaw_type::parser_indentation => { parse_msg.to_string() + separator.as_str() + &" MISSED SPACE/SEPARATOR ".bold().on_red().to_string() }
        };
        write!(f,
            "{}\n {}\n {:>5}\n {:>5}  {}\n {:>5} {}{}\n {}",
            errtype.black(),
            self.loc,
            " |".red(),
            (self.loc.row().to_string() + " |").red(),
            get_line_at(self.loc.path(), self.loc.row()).red(),
            " |".red(),
            " ".repeat(self.loc.col()),
            "^".repeat(self.loc.len()),
            self.msg,
        )
    }
}
