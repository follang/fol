#![allow(dead_code)]

use std::str::Chars;
use crate::scan::locate;
use crate::scan::reader;
use std::fmt;

/// Peekable iterator over a char sequence.
/// Next characters can be peeked via `nth` method, and position can be shifted forward via `bump` method.
pub(crate) struct PART {
    content: String,
    curr_char: char,
    prev_char: char,
}

pub(crate) const EOF_CHAR: char = '\0';


impl fmt::Display for PART {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.next_char())
    }
}

impl PART {
    pub(crate) fn init(data: &String) -> PART {
        PART {
            content: data.to_string(),
            curr_char: EOF_CHAR,
            prev_char: EOF_CHAR,
        }
    }

    /// Returns nth character relative to the current part position, if position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file, it should be checked with `is_eof` method.
    pub fn nth(&self, n: usize) -> char {
        self.content.chars().nth(n).unwrap_or(EOF_CHAR)
    }

    pub fn content(&self) -> &String {
        &self.content
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
        self.nth(0)
    }

    /// Checks if there is nothing more to consume.
    pub(crate) fn not_eof(&self) -> bool {
        !self.content.is_empty()
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self, loc: &mut locate::LOCATION) -> Option<char> {
        self.prev_char = self.curr_char;
        loc.new_char();
        if self.not_eof() {
            let c = self.content.chars().next()?;
            self.content = self.content[1..].to_string();
            self.curr_char = c;
            Some(c)
        } else {
            Some(EOF_CHAR)
        }
    }
}
