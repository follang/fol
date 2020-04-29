#![allow(dead_code)]

use std::fmt;
use std::str;
use crate::lex::locate;
use crate::lex::reader;
use crate::lex::parts;
use crate::lex::token;

pub fn is_eol(ch: &char) -> bool {
    return *ch == '\n' || *ch == '\r'
}

pub fn is_space(ch: &char) -> bool {
    return *ch == ' ' || *ch == '\t'
}

pub fn is_digit(ch: &char) -> bool {
    return '0' <= *ch && *ch <= '9'
}

pub fn is_alphabetic(ch: &char) -> bool {
    return 'a' <= *ch && *ch <= 'z' || 'A' <= *ch && *ch <= 'Z' || *ch == '_'
}

pub fn is_symbol(ch: &char) -> bool {
    return '!' <= *ch && *ch <= '/' || ':' <= *ch && *ch <= '@' || '[' <= *ch && *ch <= '^' || '{' <= *ch && *ch <= '~'
}

pub fn is_oct_digit(ch: &char) -> bool {
    return '0' <= *ch && *ch <= '7' || *ch == '_'
}
pub fn is_hex_digit(ch: &char) -> bool {
    return '0' <= *ch && *ch <= '9' || 'a' <= *ch && *ch <= 'f' || 'A' <= *ch && *ch <= 'F' || *ch == '_'
}

pub fn is_alphanumeric(ch: &char) -> bool {
    return is_digit(ch) || is_alphabetic(ch)
}

pub struct Token {
    pub tok: token::KEYWORD,
    pub loc: locate::LOCATION,
    pub len: usize,
    pub con: String,
}

impl Token {
    fn new(tok: token::KEYWORD, loc: locate::LOCATION, con: String, len: usize) -> Token {
        Token { tok, loc, con, len }
    }
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}   {}", self.loc, self.con)
    }
}

/// Creates an iterator that produces tokens from the input string.
pub fn tokenize(mut input: &str) -> impl Iterator<Item = Token> + '_ {
    let mut loc = locate::LOCATION::new(&input);
    std::iter::from_fn(move || {
        if input.is_empty() { return None; }
        let token = parts::Part::new(&input).advance_token(&mut loc);
        input = &input[token.len..];
        Some(token)
    })
}

/// Creates an iterator that produces tokens from the input string.
// pub fn reader<'a>(inp: &'a str) -> impl Iterator<Item = Token> + '_ {
    // let index: usize = 0;
    // let mut red = reader::readerize(&inp).peekable();
    // let mut input = &red.peek().unwrap().data;
    // let file = red.peek().unwrap().name.clone();
    // let mut loc = locate::LOCATION::new(&file);
    // std::iter::from_fn(move || {
        // if input.is_empty() {
            // return None;
        // }
        // let token = parts::Part::new(&input).advance_token(&mut loc);
        // input = &input[token.len..].to_string();
        // Some(token)
    // })
// }

/// Creates an iterator that produces tokens from the input string.
pub fn read_dir<'a>(inp: &'a str) -> impl Iterator<Item = Token> + '_ {
    let mut red = reader::readerize(&inp).next();
    let mut input = &red.unwrap().data;
    let mut file = &red.unwrap().name;
    let mut loc = locate::LOCATION::new(&file);
    std::iter::from_fn(move || {
        if input.is_empty() {
            if red == None {
                return None
            } else {
                red = reader::readerize(&inp).next();
                input = &red.unwrap().data;
                file = &red.unwrap().name;
                loc = locate::LOCATION::new(&file);
                let tok = Token::new(token::KEYWORD::void_(token::VOID::endfile_), loc.clone(), "".to_string(), 0);
                return Some(tok)
            }
        }
        let token = parts::Part::new(&input).advance_token(&mut loc);
        input = &input[token.len..].to_string();
        Some(token)
    })
}

use crate::lex::token::*;
use crate::lex::token::KEYWORD::*;
impl parts::Part<'_> {
    /// Parses a token from the input string.
    fn advance_token(&mut self, loc: &mut locate::LOCATION) -> Token {
        let tok = assign_(ASSIGN::var_);
        let first_char = self.bumpit(loc).unwrap();
        let result = Token::new(tok, loc.clone(), self.curr_char().to_string(), 1);
        if is_eol(&first_char) {
            loc.new_line()
        }
        return result
    }

    fn eat_while<F>(&mut self, mut predicate: F) -> usize
    where
        F: FnMut(char) -> bool,
    {
        let mut eaten: usize = 0;
        while predicate(self.first()) && !self.is_eof() {
            eaten += 1;
            self.bump();
        }
        eaten
    }
}
