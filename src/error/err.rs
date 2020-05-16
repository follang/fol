#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use std::fmt;
use crate::scan::locate;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error;
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct error {
    typ: TYPE,
    msg: String,
    loc: locate::LOCATION,
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ERROR {
    el: Vec<error>
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TYPE {
    lexer,
    parser,
}
impl error {
    pub fn new(typ: TYPE, msg: &str, loc: locate::LOCATION) -> Self {
        error { typ, msg: msg.to_string(), loc }
    }
}

impl ERROR {
    pub fn init() -> Self {
        ERROR{ el: Vec::new() }
    }

    pub fn list(&self) -> &Vec<error> {
        &self.el
    }

    pub fn report(&mut self, typ: TYPE, msg: &str, loc:locate::LOCATION) {
        let e = error::new(typ, msg, loc);
        &self.el.push(e);
    }

    pub fn show(&mut self) {
        println!("\n");
        for e in self.el.iter() {
            println!("{}", e);
            println!("\n--------------------------------------------\n");
        }
    }
}

fn get_line_at(filepath: &str, line_num: usize) -> Result<String, Error> {
    let path = Path::new(filepath);
    let file = File::open(path).expect("File not found or cannot be opened");
    let content = BufReader::new(&file);
    let mut lines = content.lines();
    lines.nth(line_num-1).expect("No line found at that position")
}

impl fmt::Display for error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.typ {
            TYPE::lexer => write!(f,
                "{}\n{} \n\n{} \n{}\n{}",
                "error in lexing stage:",
                self.loc,
                get_line_at(self.loc.path(), self.loc.row()).unwrap(),
                "^^^",
                self.msg,
                ),
            TYPE::parser => write!(f,
                "{}\n{} \n\n{} \n{}\n{}",
                "error in parsing stage:",
                self.loc,
                get_line_at(self.loc.path(), self.loc.row()).unwrap(),
                "^^^",
                self.msg,
                ),
        }
    }
}
