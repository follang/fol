#![allow(unused_variables)]
#![allow(dead_code)]

use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use colored::Colorize;

use crate::types::*;
use crate::syntax::index::source::Source;


/// A Point somewhere in the source code.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    row: usize,
    col: usize,
    len: usize,
    deep: isize,
}

impl std::default::Default for Point {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            len: 0,
            deep: 0,
        }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[{: <2}:{: <2}]",
            self.row, self.col
        )
    }
}

impl Point {
    pub fn visualize(&self, source: &Source) -> String {
        let file = File::open(Path::new(&source.path(true))).unwrap();
        let mut lines = BufReader::new(&file).lines();
        let line = lines.nth(self.row() - 1).unwrap().unwrap();
        format!(
            "{}\n {:>6}\n {:>6}  {}\n {:>6} {}{}",
            self.print(source),
            " |".red(),
            (self.row().to_string() + " |").red(),
            line.red(),
            " |".red(),
            " ".repeat(self.col()),
            "^".repeat(self.len()),
        )
    }

    pub fn print(&self, source: &Source) -> String {
        format!(
            "{: <4} {}",
            source.path(false), self
        )
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

    pub fn set_len(&mut self, l: usize) {
        self.len = l
    }

    pub fn longer(&mut self, i: &usize) {
        self.len += i
    }

    pub fn deep(&self) -> isize {
        self.deep
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

    pub fn adjust(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }

    pub fn deepen(&mut self) {
        self.deep += 1
    }

    pub fn soften(&mut self) {
        self.deep -= 1
    }
}
