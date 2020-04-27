#![allow(dead_code)]

use std::str::Chars;
use std::fmt;

/// Peekable iterator over a char sequence.
///
/// Next characters can be peeked via `nth_char` method,
/// and position can be shifted forward via `bump` method.
pub(crate) struct Part<'a> {
    initial_len: usize,
    restof: Chars<'a>,
    prev: char,
}

pub(crate) const EOF_CHAR: char = '\0';


impl fmt::Display for Part<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.first())
    }
}

impl<'a> Part<'a> {
    pub(crate) fn new(input: &'a str) -> Part<'a> {
        Part {
            initial_len: input.len(),
            restof: input.chars(),
            prev: EOF_CHAR,
        }
    }

    /// For debug assertions only
    /// Returns the last eaten symbol (or '\0' in release builds).
    pub(crate) fn prev(&self) -> char {
        self.prev
    }

    /// Returns nth character relative to the current part position.
    /// If requested position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file,
    /// it should be checked with `is_eof` method.
    fn nth_char(&self, n: usize) -> char {
        self.restof().nth(n).unwrap_or(EOF_CHAR)
    }

    /// Peeks the next symbol from the input stream without consuming it.
    pub(crate) fn first(&self) -> char {
        self.nth_char(0)
    }

    /// Peeks the second symbol from the input stream without consuming it.
    pub(crate) fn second(&self) -> char {
        self.nth_char(1)
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
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.restof.next()?;
        self.prev = c;
        Some(c)
    }
}
