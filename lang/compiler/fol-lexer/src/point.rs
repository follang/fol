#![allow(unused_variables)]
#![allow(dead_code)]

use colored::Colorize;
use std::fmt;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
// Simple Source type for location tracking
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Source {
    path: String,
}

impl Source {
    pub fn new(path: String) -> Self {
        Self { path }
    }

    pub fn path(&self, _abs: bool) -> String {
        self.path.clone()
    }
}

/// A location somewhere in the sourcecode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Location {
    row: usize,
    col: usize,
    len: usize,
    deep: isize,
    src: Option<Source>,
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{: <2}:{: <2} | {}]", self.row, self.col, self.deep)
    }
}

impl Location {
    pub fn visualize(&self, src: &Option<Source>) -> String {
        if let Some(source) = src {
            match File::open(Path::new(&source.path(true))) {
                Ok(file) => {
                    let mut lines = BufReader::new(&file).lines();
                    match lines.nth(self.row().saturating_sub(1)) {
                        Some(Ok(line)) => format!(
                            "{}\n {:>6}\n {:>6}  {}\n {:>6} {}{}",
                            self.print(source),
                            " |".red(),
                            (self.row().to_string() + " |").red(),
                            line.red(),
                            " |".red(),
                            " ".repeat(self.col()),
                            "^".repeat(self.len()),
                        ),
                        Some(Err(_)) | None => format!(
                            "{}\n {:>6}\n {:>6}  <source line unavailable>\n {:>6} {}{}",
                            self.print(source),
                            " |".red(),
                            (self.row().to_string() + " |").red(),
                            " |".red(),
                            " ".repeat(self.col()),
                            "^".repeat(self.len()),
                        ),
                    }
                }
                Err(_) => format!(
                    "{}\nsource file unavailable: {}",
                    self.print(source),
                    source.path(true)
                ),
            }
        } else {
            format!("at line: {:>6}", self,)
        }
    }

    pub fn print(&self, source: &Source) -> String {
        format!("{: <4} {}", source.path(false), self)
    }

    pub fn set_source(&mut self, source: &Option<Source>) {
        self.src = source.clone()
    }

    pub fn from_stream_location(stream_loc: &fol_stream::Location) -> Self {
        Self {
            row: stream_loc.row,
            col: stream_loc.col,
            len: 1,
            deep: 0,
            src: stream_loc.file.as_ref().map(|f| Source::new(f.clone())),
        }
    }

    pub fn source(&self) -> Option<Source> {
        self.src.clone()
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

    pub fn is_empty(&self) -> bool {
        self.len == 0
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

    pub fn set_deep(&mut self, deep: isize) {
        self.deep = deep
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
}
