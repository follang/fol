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
    lexer(lexer),
    parser(parser)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum parser {
    parser_unexpected,
    parser_missmatch,
    parser_space_rem,
    parser_space_add,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum lexer {
    lexer_bracket_unmatch,
    lexer_space_add,
    lexer_primitive_access,
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
            println!("\n\n{}{:<2} >> {}", " FLAW #".black().on_red(), (&i + 1).to_string().black().on_red(), e);
        }
        if self.el.len() != 0 {
            let num = if self.el.len() == 1 { "flaw" } else { "flaws" };
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
        write!(f,
            "{}\n {}\n {:>5}\n {:>5}  {}\n {:>5} {}{}\n {}",
            self.typ,
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

impl fmt::Display for flaw_type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value: String = match self {
            flaw_type::lexer(a) => " lexing stage ".black().bold().on_white().to_string() + ":" + a.to_string().as_str(),
            flaw_type::parser(b) => " parsing stage ".black().bold().on_white().to_string() + ":" + b.to_string().as_str()
        };
        write!(f, "{}", value)
    }
}

impl fmt::Display for parser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value: String = match self {
            parser::parser_unexpected => " UNEXPECTED TOKEN ".to_string(),
            parser::parser_missmatch => " MISSMATCHED ARGUMENTS ".to_string(),
            parser::parser_space_add => " MISSING BLANK SPACE ".to_string(),
            parser::parser_space_rem => " OBSOLETE BLANK SPACE ".to_string(),
        };
        write!(f, "{}", value.on_red().to_string())
    }
}

impl fmt::Display for lexer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value: String = match self {
            lexer::lexer_bracket_unmatch => " UNMATCHED BRACKET ".to_string(),
            lexer::lexer_space_add => " MISSING BLANK SPACE ".to_string(),
            lexer::lexer_primitive_access => " PRIMITIVE_ACCESS ".to_string(),
        };
        write!(f, "{}", value.on_red().to_string())
    }
}
