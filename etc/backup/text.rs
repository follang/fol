#![allow(dead_code)]

use std::fmt;
use std::str::Chars;
use crate::syntax::point;
use crate::syntax::scan::source;

use crate::syntax::error::*;

pub const EOF_CHAR: char = '\0';
const SLIDER: usize = 6;

/// Peekable iterator over a char sequence.
/// Next characters can be peeked via `nth` method, and position can be shifted forward via `bump` method.
pub struct Text {
    lines: Box<dyn Iterator<Item = String>>,
    chars: Box<dyn Iterator<Item = char>>,
    win: (Vec<char>, char, Vec<char>),
    _in_count: usize,
}

fn lines(src: source::Source) -> impl Iterator<Item = String> {
    let mut reader = reader::BufReader::open(src.path(true)).unwrap();
    let mut buffer = String::new();
    std::iter::from_fn(move || {
        if let Some(line) = reader.read_line(&mut buffer) {
            return Some(line.unwrap().clone());
        }
        None
    })
}

fn chars(src: String) -> impl Iterator<Item = char> + 'static {
    let mut chrs = src.clone();
    std::iter::from_fn(move || {
        if let Some(ch) =  chrs.chars().next() {
            chrs.remove(0);
            return Some(ch.clone()) 
        };
        None
    })
}


impl Text {
    pub fn init(src: source::Source) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut lines = Box::new(lines(src.clone()));
        let mut chars = Box::new(chars(lines.next().unwrap()));
        for _ in 0..SLIDER { prev.push('\n') }
        for _ in 0..SLIDER { next.push(chars.next().unwrap()) }
        Self {
            chars,
            lines,
            win: (prev, '\n', next),
            _in_count: SLIDER
        }
    }
    pub fn curr(&self) -> char {
        self.win.1
    }
    pub fn next_vec(&self) -> Vec<char> {
        self.win.2.clone()
    }
    pub fn next(&self) -> char { 
        self.next_vec()[0] 
    }
    pub fn prev_vec(&self) -> Vec<char> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn prev(&self) -> char { 
        self.prev_vec()[0] 
    }
    pub fn bump(&mut self, loc: &mut point::Location) -> Opt<char> {
        match self.chars.next() {
            Some(v) => {
                loc.new_char();
                self.win.0.remove(0); self.win.0.push(self.win.1);
                self.win.1 = self.win.2[0];
                self.win.2.remove(0); self.win.2.push(v);
                return Some(self.win.1)
            },
            None => {
                match self.lines.next() {
                    Some(v) => {
                        self.chars = Box::new(chars(v.clone()));
                        loc.new_char();
                        return Some('\n')
                    },
                    None => {
                        if self._in_count > 1 {
                            self.win.0.remove(0); self.win.0.push(self.win.1);
                            self.win.1 = self.win.2[0];
                            self.win.2.remove(0); self.win.2.push('\n');
                            self._in_count -= 1;
                            return Some(self.win.1)
                        } else { return None }
                    }
                }
            }
        }
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.win.1)
    }
}


mod reader {
    use std::{
        fs::File,
        io::{self, prelude::*},
    };

    pub struct BufReader {
        reader: io::BufReader<File>,
    }

    impl BufReader {
        pub fn open(path: impl AsRef<std::path::Path>) -> io::Result<Self> {
            let file = File::open(path)?;
            let reader = io::BufReader::new(file);

            Ok(Self { reader })
        }

        pub fn read_line<'buf>(
            &mut self,
            buffer: &'buf mut String,
        ) -> Option<io::Result<&'buf mut String>> {
            buffer.clear();

            self.reader
                .read_line(buffer)
                .map(|u| if u == 0 { None } else { Some(buffer) })
                .transpose()
        }
    }
}


//  pub fn chars(src: source::Source) -> impl Iterator<Item = char> {
//      use std::fs::File;
//      use std::io::{self, prelude::*, BufReader};

//      let file = File::open(src.path(true)).unwrap();
//      let reader = BufReader::new(file);
//      let line = reader.lines().next();
//      std::iter::from_fn(move || {
//          for line2 in reader.lines() {
//              println!("{}", line2.unwrap());
//          }
//          None
//      })
//  }

