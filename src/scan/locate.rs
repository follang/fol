#![allow(unused_variables)]
#![allow(dead_code)]

use crate::scan::reader;
use std::fmt;

/// A location somewhere in the sourcecode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LOCATION {
    ns: String,
    pos: isize,
    row: usize,
    col: usize,
    len: usize,
    deep: isize,
}

impl fmt::Display for LOCATION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ns: {: <5}   pos: {: <3}   row: {: <3}   col: {: <3}   deep: {: <3}  len: {: <5}",
            self.ns, self.pos, self.row, self.col, self.deep, self.len)
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
    pub fn new(red: &reader::READER) -> Self {
        LOCATION { ns: red.name.to_string(), pos: 1, row: 1, col: 1, len: 1, deep: 1 }
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

    pub fn deep(&self) -> isize {
        self.deep
    }

    pub fn pos(&self) -> isize {
        self.pos
    }

    pub fn ns(&self) -> &String {
        &self.ns
    }

    pub fn reset(&mut self) {
        self.row = 1;
        self.col = 1;
        self.len = 1;
        self.pos = 1;
    }

    pub fn new_char(&mut self) {
        self.col += 1;
        self.len += 1;
        self.pos += 1;
    }

    pub fn new_word(&mut self) {
        self.len = 0;
    }

    pub fn new_line(&mut self) {
        self.row += 1;
        self.col = 1;
    }

    pub fn adjust(&mut self, row: usize, col: usize, pos: isize){
        self.row = row;
        self.col = col;
        self.pos = pos;
    }

    pub fn deepen(&mut self){
        self.deep += 1
    }

    pub fn soften(&mut self){
        self.deep -= 1
    }
}
