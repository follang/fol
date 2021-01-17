#![allow(dead_code)]

use std::fmt;
use std::str::Chars;
use crate::syntax::point;

/// Peekable iterator over a char sequence.
/// Next characters can be peeked via `nth` method, and position can be shifted forward via `bump` method.
pub(crate) struct Text {
    fulltext: String,
    curr_char: char,
    prev_char: char,
}

pub(crate) const EOF_CHAR: char = '\0';

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.next_char())
    }
}

impl Text {
    pub(crate) fn init(data: &String) -> Text {
        Text {
            fulltext: data.to_string(),
            curr_char: EOF_CHAR,
            prev_char: EOF_CHAR,
        }
    }

    /// Returns nth character relative to the current position, if position doesn't exist then `EOF_CHAR` is returned.
    pub fn nth(&self, n: usize) -> char {
        self.fulltext.chars().nth(n).unwrap_or(EOF_CHAR)
    }

    pub fn fulltext(&self) -> &String {
        &self.fulltext
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
        !self.fulltext.is_empty()
    }

    /// Moves to the next character.
    pub(crate) fn bump(&mut self, loc: &mut point::Location) -> Option<char> {
        self.prev_char = self.curr_char;
        loc.new_char();
        if self.not_eof() {
            let c = self.fulltext.chars().next()?;
            self.fulltext = self.fulltext[1..].to_string();
            self.curr_char = c;
            Some(c)
        } else {
            Some(EOF_CHAR)
        }
    }
}
