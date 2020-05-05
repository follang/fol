#![allow(dead_code)]

use std::str::Chars;
use crate::scan::locate;
use crate::scan::reader;
use std::fmt;

/// Peekable iterator over a char sequence.
/// Next characters can be peeked via `peek` method, and position can be shifted forward via `bump` method.
pub(crate) struct PART<'a> {
    initial_len: usize,
    content: String,
    restof: Chars<'a>,
    pastof: Chars<'a>,
    curr_char: char,
    prev_char: char,
}

pub(crate) const EOF_CHAR: char = '\0';


impl fmt::Display for PART<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.next_char())
    }
}

impl<'a> PART<'a> {
    pub(crate) fn init(red: &'a reader::READER) -> PART<'a> {
        PART {
            initial_len: red.data.len(),
            content: String::new(),
            restof: red.data.chars(),
            pastof: red.past.chars(),
            curr_char: EOF_CHAR,
            prev_char: red.past.chars().rev().nth(0).unwrap_or(EOF_CHAR),
        }
    }

    /// Returns peek character relative to the current part position, if position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file, it should be checked with `is_eof` method.
    pub fn peek(&self, n: usize) -> char {
        self.restof().nth(n).unwrap_or(EOF_CHAR)
    }

    /// Returns the last eaten symbol
    pub(crate) fn curr_char(&self) -> char {
        self.curr_char
    }

    /// Returns the past eaten symbol
    pub(crate) fn prev_char(&self) -> char {
        self.prev_char
    }

    /// Peeks the next symbol from the input stream without consuming it.
    pub(crate) fn next_char(&self) -> char {
        self.peek(0)
    }

    /// Returns the content of the part/chunk
    pub(crate) fn content(&self) -> &String {
        &self.content
    }

    /// Adds a character the content of the part/chunk
    pub(crate) fn concat(&mut self, c: char) -> &str {
        self.content.push_str(&c.to_string());
        self.content.as_str()
    }

    /// Checks if there is nothing more to consume.
    pub(crate) fn is_eof(&self) -> bool {
        self.restof.as_str().is_empty()
    }

    /// Returns amount of already consumed symbols.
    pub(crate) fn len_consumed(&self) -> usize {
        self.initial_len - self.restof.as_str().len()
    }

    /// Returns a `Chars` iterator over the remaining characters.
    fn restof(&self) -> Chars<'a> {
        self.restof.clone()
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self, loc: &mut locate::LOCATION) -> Option<char> {
        self.prev_char = self.curr_char().clone();
        println!("(p){}", self.curr_char);
        let c = self.restof.next()?;
        self.curr_char = c;
        loc.new_char();
        Some(c)
    }
}
