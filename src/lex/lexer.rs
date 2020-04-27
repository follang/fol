#![allow(dead_code)]

use std::fmt;
use crate::lex::locate;
use crate::lex::reader;
use crate::lex::parts;
use crate::lex::token;

pub fn is_eol(ch: char) -> bool {
    return ch == '\n' || ch == '\r'
}

pub fn is_space(ch: char) -> bool {
    return ch == ' ' || ch == '\t'
}

pub fn is_digit(ch: char) -> bool {
    return '0' <= ch && ch <= '9'
}

pub fn is_alphabetic(ch: char) -> bool {
    return 'a' <= ch && ch <= 'z' || 'A' <= ch && ch <= 'Z' || ch == '_'
}

pub fn is_symbol(ch: char) -> bool {
    return '!' <= ch && ch <= '/' || ':' <= ch && ch <= '@' || '[' <= ch && ch <= '^' || '{' <= ch && ch <= '~'
}

pub fn is_oct_digit(ch: char) -> bool {
    return '0' <= ch && ch <= '7' || ch == '_'
}
pub fn is_hex_digit(ch: char) -> bool {
    return '0' <= ch && ch <= '9' || 'a' <= ch && ch <= 'f' || 'A' <= ch && ch <= 'F' || ch == '_'
}

pub fn is_alphanumeric(ch: char) -> bool {
    return is_digit(ch) || is_alphabetic(ch)
}

pub struct Token {
    pub tok: token::KEYWORD,
    pub loc: locate::Location,
    pub len: usize,
}

impl Token {
    fn new(tok: token::KEYWORD, loc: locate::Location, len: usize) -> Token {
        Token { tok, loc, len }
    }
}

/// Parses the first token from the provided input string.
pub fn first_token(input: &str) -> Token {
    debug_assert!(!input.is_empty());
    parts::Part::new(input).advance_token()
}

/// Creates an iterator that produces tokens from the input string.
pub fn tokenize(mut input: &str) -> impl Iterator<Item = Token> + '_ {
    std::iter::from_fn(move || {
        if input.is_empty() { return None; }
        let token = first_token(input);
        input = &input[token.len..];
        Some(token)
    })
}

impl parts::Part<'_> {
    /// Parses a token from the input string.
    fn advance_token(&mut self) -> Token {
        let tok = token::KEYWORD::assign_(token::ASSIGN::var_);
        let loc = locate::Location::new("sdsds", 1, 1);
        Token{ tok, loc, len: 5 }
    }
}
