#![allow(unused_variables)]
#![allow(dead_code)]

use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use colored::Colorize;


/// A location somewhere in the sourcecode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Location {
    module: String,
    path: (String, String),
    row: usize,
    col: usize,
    len: usize,
    deep: isize,
}

impl std::default::Default for Location {
    fn default() -> Self {
        Self {
            path: (String::new(), String::new()),
            module: String::new(),
            row: 0,
            col: 0,
            len: 0,
            deep: 1,
        }
    }
}

impl Location {
    pub fn visualize(&self) -> String {
        let file = File::open(Path::new(&self.path.1)).unwrap();
        let mut lines = BufReader::new(&file).lines();
        let line = lines.nth(self.row() - 1).unwrap().unwrap();
        format!(
            "{}\n {:>6}\n {:>6}  {}\n {:>6} {}{}",
            self,
            " |".red(),
            (self.row().to_string() + " |").red(),
            line.red(),
            " |".red(),
            " ".repeat(self.col()),
            "^".repeat(self.len()),
        )
    }

    pub fn init(path: (String, String), module: &String) -> Self {
        Self { path: path, module: module.to_string(), ..Default::default() }
    }

    pub fn new(
        path: (String, String),
        module: String,
        row: usize,
        col: usize,
        len: usize,
        deep: isize,
    ) -> Self {
        Location {
            path,
            module,
            row,
            col,
            len,
            deep,
        }
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

    pub fn path(&self, abs: bool) -> &String {
        if abs { &self.path.0 } else { &self.path.1 }
    }

    pub fn module(&self) -> &String {
        &self.module
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

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "file: {: <4}   row: {: <2}   col: {: <2}",
            self.path(false), self.row, self.col
        )
    }
}

