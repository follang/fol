#![allow(unused_variables)]
#![allow(dead_code)]

use crate::scan::reader;
use std::fmt;

/// A location somewhere in the sourcecode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LOCATION {
    file: String,
    row: usize,
    col: usize,
    deep: isize,
}

impl fmt::Display for LOCATION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file: {: <5}   row: {: <3}   col: {: <3}   deep: {: <3}",
            self.file, self.row, self.col, self.deep)
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
        let file = red.path.to_string();
        // file.add_str("go");
        LOCATION { file, row: 1, col: 1, deep: 1 }
    }
    pub fn def() -> Self {
        LOCATION { file: String::new(), row: 1, col: 1, deep: 1 }
    }
    pub fn new(file: String, row: usize, col: usize, deep: isize) -> Self {
        LOCATION { file, row, col, deep }
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn deep(&self) -> isize {
        self.deep
    }

    pub fn file(&self) -> &String {
        &self.file
    }

    pub fn reset(&mut self) {
        self.row = 1;
        self.col = 1;
    }

    pub fn new_char(&mut self) {
        self.col += 1;
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 1;
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
