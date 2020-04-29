#![allow(unused_variables)]
#![allow(dead_code)]

use std::fmt;

/// A location somewhere in the sourcecode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LOCATION {
    file: String,
    row: usize,
    col: usize,
    len: usize,
}

impl fmt::Display for LOCATION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file: {}, row: {}, col: {}", self.file, self.row, self.col)
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
    pub fn new(file: &str) -> Self {
        LOCATION { file: file.to_string(), row: 1, col: 0, len: 0 }
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

    pub fn file(&self) -> &String {
        &self.file
    }

    pub fn reset(&mut self) {
        self.row = 1;
        self.col = 0;
    }

    pub fn set_len(&mut self, i: usize) {
        self.len = i;
    }

    pub fn new_char(&mut self) {
        self.col += 1;
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 0;
    }

    pub fn new_file(&mut self, s: String) {
        self.file = s;
        self.reset();
    }
}
