#![allow(unused_variables)]
#![allow(dead_code)]

use crate::scan::reader;
use std::fmt;

/// A location somewhere in the sourcecode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LOCATION {
    path: String,
    name: String,
    row: usize,
    col: usize,
    len: usize,
    deep: isize,
}

impl fmt::Display for LOCATION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file: {: <4}   row: {: <2}   col: {: <2}",
            self.name, self.row, self.col)
    }
}

impl LOCATION {
    pub fn visualize(&self, desc: &str) -> String {
        format!(
            "{}↑\n{}{}",
            " ".repeat(self.col - 1),
            " ".repeat(self.col - 1),
            desc
        )
    }
}

impl LOCATION {
    pub fn init(red: &reader::READER) -> Self {
        let name = red.name.to_string();
        let path = red.path.to_string();
        // file.add_str("go");
        LOCATION { path, name,  row: 1, col: 1, len: 1, deep: 1 }
    }
    pub fn def() -> Self {
        LOCATION { path: String::new(), name: String::new(), row: 1, col: 1, len: 1, deep: 1 }
    }
    pub fn new(path: String, name: String, row: usize, col: usize, len: usize, deep: isize) -> Self {
        LOCATION { path, name, row, col, len, deep }
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn longer(&mut self, i: &usize) {
        self.len += i
    }

    pub fn deep(&self) -> isize {
        self.deep
    }

    pub fn path(&self) -> &String {
        &self.path
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn reset(&mut self) {
        self.row = 1;
        self.col = 1;
        self.len = 1;
    }

    pub fn new_char(&mut self) {
        self.col += 1;
        self.len += 1;
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn new_word(&mut self) {
        self.len = 0;
    }

    pub fn adjust(&mut self, row: usize, col: usize){
        self.row = row;
        self.col = col;
    }

    pub fn deepen(&mut self){
        self.deep += 1
    }

    pub fn soften(&mut self){
        self.deep -= 1
    }
}
