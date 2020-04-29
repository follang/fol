#![allow(dead_code)]

use std::str::Chars;
use crate::lex::locate;
use std::fmt;

/// Peekable iterator over a char sequence.
/// Next characters can be peeked via `nth_char` method, and position can be shifted forward via `bump` method.
pub(crate) struct Part<'a> {
    initial_len: usize,
    content: String,
    restof: Chars<'a>,
    curr_char: char,
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
            content: String::new(),
            restof: input.chars(),
            curr_char: EOF_CHAR,
        }
    }

    /// Returns the last eaten symbol
    pub(crate) fn curr_char(&self) -> char {
        self.curr_char
    }

    /// Returns nth character relative to the current part position, if position doesn't exist, `EOF_CHAR` is returned.
    /// However, getting `EOF_CHAR` doesn't always mean actual end of file, it should be checked with `is_eof` method.
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
    pub(crate) fn bump(&mut self) -> Option<char> {
        let c = self.restof.next()?;
        self.curr_char = c;
        Some(c)
    }

    pub(crate) fn bumpit(&mut self, loc: &mut locate::LOCATION) -> Option<char> {
        let c = self.restof.next()?;
        self.curr_char = c;
        loc.new_char();
        Some(c)
    }
}
