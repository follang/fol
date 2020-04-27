#![allow(unused_variables)]
#![allow(dead_code)]

use std::fmt;

/// A location somewhere in the sourcecode.
pub struct Location {
    file: String,
    row: usize,
    col: usize,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "file: {}, row: {}, col: {}", self.file, self.row, self.col)
    }
}

impl Location {
    pub fn visualize(&self, desc: &str) -> String {
        format!(
            "{}↑\n{}{}",
            " ".repeat(self.col - 1),
            " ".repeat(self.col - 1),
            desc
        )
    }
}

impl Location {
    pub fn new(file: &str, row: usize, col: usize) -> Self {
        Location { file: file.to_string(), row, col }
    }

    pub fn row(&self) -> usize {
        self.row
    }

    pub fn col(&self) -> usize {
        self.col
    }

    pub fn file(&self) -> &String {
        &self.file
    }

    pub fn reset(&mut self) {
        self.row = 1;
        self.col = 1;
    }

    pub fn go_right(&mut self) {
        self.col += 1;
    }

    pub fn newline(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn newfile(&mut self, s: String) {
        self.file = s;
        self.reset();
    }
}
