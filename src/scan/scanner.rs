#![allow(dead_code)]

use std::fmt;
use std::str;
use crate::scan::locate;
use crate::scan::reader;
use crate::scan::parts;
use crate::scan::token;

pub fn is_eol(ch: &char) -> bool {
    return *ch == '\n' || *ch == '\r'
}

pub fn is_space(ch: &char) -> bool {
    return *ch == ' ' || *ch == '\t'
}

pub fn is_digit(ch: &char) -> bool {
    return '0' <= *ch && *ch <= '9'
}

pub fn is_alpha(ch: &char) -> bool {
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
    return is_digit(ch) || is_alpha(ch)
}

pub fn is_void(ch: &char) -> bool {
    return is_eol(ch) || is_space(ch)
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SCAN {
    key: token::KEYWORD,
    loc: locate::LOCATION,
    con: String,
}

impl SCAN {
    fn new(key: token::KEYWORD, loc: locate::LOCATION, con: String) -> Self {
        SCAN { key, loc, con }
    }
}

impl SCAN {
    pub fn key(&self) -> &token::KEYWORD {
        &self.key
    }
    pub fn loc(&self) -> &locate::LOCATION {
        &self.loc
    }
    pub fn con(&self) -> &String {
        &self.con
    }
}

impl fmt::Display for SCAN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {: <20} {}", self.loc, self.key, self.con)
    }
}

/// Creates an iterator that produces tokens from the input string.
// pub fn tokenize(mut input: &str) -> impl Iterator<Item = SCAN> + '_ {
    // let mut loc = locate::LOCATION::new(&input);
    // std::iter::from_fn(move || {
        // if input.is_empty() { return None; }
        // let token = parts::PART::new(&input).scan(&mut loc);
        // input = &input[token.loc.len()..];
        // Some(token)
    // })
// }

// pub fn reader2<'a, I>(mut inp: I) -> impl Iterator<Item = SCAN> +'a
// where
    // I: Iterator<Item = &'a reader::READER>,
// {
    // let red = inp.by_ref().next();
    // let mut loc = locate::LOCATION::new(&red.unwrap().name);
    // std::iter::from_fn(move || {
        // if red.unwrap().data.is_empty() {
            // return None;
        // }
        // let token = parts::PART::new(&red.unwrap().data).scan(&mut loc);
        // let a = red.unwrap().data[token.len..].to_string();
        // red.unwrap().set(a);
        // Some(token)
    // })
// }



/// Creates a vector that produces tokens from the input string.
pub fn vectorize(red: &mut reader::READER) -> Vec<SCAN> {
    let mut vec: Vec<SCAN> = Vec::new();
    let mut loc = locate::LOCATION::new(&red);
    while !red.data.is_empty() {
        let token = parts::PART::init(&red).scan(&mut loc);
        red.past = red.data[..token.loc.len()].to_string();
        red.data = red.data[token.loc.len()..].to_string();
        vec.push(token)
    }
    vec
}

/// Creates an iterator that produces tokens from the input string.
// pub fn iteratize(red: &mut reader::READER) -> impl Iterator<Item = SCAN> + '_ {
    // let mut loc = locate::LOCATION::new(&red.path);
    // std::iter::from_fn(move || {
        // if red.data.is_empty() {
            // return None;
        // }
        // let token = parts::PART::init(&red.data).scan(&mut loc);
        // red.data = red.data[token.loc.len()..].to_string();
        // Some(token)
    // })
// }

use crate::scan::token::*;
use crate::scan::token::KEYWORD::*;
impl parts::PART<'_> {
    fn eat_while<F>(&mut self, mut predicate: F) -> usize
    where
        F: FnMut(char) -> bool,
        {
            let mut eaten: usize = 0;
            while predicate(self.next_char()) && !self.is_eof() {
                eaten += 1;
            }
            eaten
        }

    /// Parses a token from the input string.
    fn scan(&mut self, loc: &mut locate::LOCATION) -> SCAN {
        let mut result = SCAN::new(illegal, loc.clone(), String::new());
        result.loc.new_word();
        self.bump(&mut result.loc);
        if is_eol(&self.curr_char()) {
            result.endline(self, false);
        } else if is_space(&self.curr_char()) {
            result.space(self);
        } else if self.curr_char() == '"' || self.curr_char() == '\'' || self.curr_char() == '`' {
            result.encap(self);
        } else if is_digit(&self.curr_char()) {
            result.digit(self);
        } else if is_symbol(&self.curr_char()) {
            result.symbol(self);
        } else if is_alpha(&self.curr_char()) {
            result.alpha(self);
        }
        let (row, col, pos) = (loc.row(), loc.col(), loc.pos());
        *loc = result.loc.clone();
        result.loc.adjust(row, col, pos);
        return result
    }
}

impl SCAN {
    fn endline(&mut self, part: &mut parts::PART, terminated: bool) {
        self.push_curr(part);
        self.loc.new_line();
        self.key = void(VOID::endline_ {terminated});
        while is_eol(&part.next_char()) || is_space(&part.next_char()) {
            if is_eol(&part.next_char()) { self.loc.new_line(); }
            self.bump_next(part);
        }
        // self.con = " ".to_string();
    }
    fn space(&mut self, part: &mut parts::PART) {
        self.push_curr(part);
        while is_space(&part.next_char()) {
            self.bump_next(part);
        }
        if is_eol(&part.next_char()) {
            self.bump_next(part);
            self.endline(part, false);
            return
        }
        self.key = void(VOID::space_);
        self.con = " ".to_string();
    }
    fn digit(&mut self, part: &mut parts::PART) {
        if part.curr_char() == '.' {
            self.push_curr(part);
            self.key = literal(LITERAL::float_);
            while is_digit(&part.next_char()) || part.next_char() == '_' { self.bump_next(part); }
        } else if part.curr_char() == '0' && ( part.next_char() == 'x' || part.next_char() == 'o' || part.next_char() == 'b' ) {
            self.push_curr(part);
            if part.next_char() == 'x' {
                self.bump_next(part);
                self.key = literal(LITERAL::hexal_);
                while is_hex_digit(&part.next_char()) { self.bump_next(part); }
            } else if part.next_char() == 'o' {
                self.bump_next(part);
                self.key = literal(LITERAL::octal_);
                while is_oct_digit(&part.next_char()) { self.bump_next(part); }
            } else if part.next_char() == 'b' {
                self.bump_next(part);
                self.key = literal(LITERAL::octal_);
                while part.next_char() == '0' || part.next_char() == '1' || part.next_char() == '_' { self.bump_next(part); }
            }
        } else {
            self.push_curr(part);
            self.key = literal(LITERAL::decimal_);
            while is_digit(&part.next_char()) || part.next_char() == '_' { self.bump_next(part); }
            if part.next_char() == '.' && is_digit(&part.nth(1)) {
                self.bump_next(part);
                if is_digit(&part.next_char()) {
                    self.key = literal(LITERAL::float_);
                    while is_digit(&part.next_char()) || part.next_char() == '_' { self.bump_next(part); }
                } else if is_void(&part.next_char()) {
                    self.key = literal(LITERAL::float_);
                    while is_digit(&part.next_char()) || part.next_char() == '_' { self.bump_next(part); }
                }
            }
        }
        // if part.next_char() == '.' && !(is_alpha(&part.nth(1)) || (is_eol(&part.nth(1)) && is_alpha(&part.nth(2)))) {
        if part.next_char() == '.' && is_digit(&part.nth(1)) {
            self.key = illegal;
            while !is_void(&part.next_char()) { self.bump_next(part); }
        }
    }
    fn encap(&mut self, part: &mut parts::PART) {
        let litsym = part.curr_char();
        if litsym == '`' { self.key = comment;}
        else if litsym == '\'' { self.key = literal(LITERAL::char_); }
        else { self.key = literal(LITERAL::string_); }
        self.push_curr(part);
        while part.next_char() != litsym || (part.next_char() == litsym && part.curr_char() == '\\') {
            if part.next_char() != litsym && part.next_char() == '\0' { self.key = illegal; break }
            else if is_eol(&part.next_char()) { self.loc.new_line(); }
            self.bump_next(part);
        }
        self.bump_next(part);
    }
    fn symbol(&mut self, part: &mut parts::PART) {
        if (part.curr_char() == '.' || part.curr_char() == '-') && is_digit(&part.next_char()) {
            self.digit(part);
            return
        }
        self.push_curr(part);
        self.key = symbol(SYMBOL::curlyC_);
        match part.curr_char() {
            '{' => { self.loc.deepen(); self.key = symbol(SYMBOL::curlyO_) },
            '}' => { self.loc.soften(); self.key = symbol(SYMBOL::curlyC_) },
            '[' => { self.loc.deepen(); self.key = symbol(SYMBOL::squarO_) },
            ']' => { self.loc.soften(); self.key = symbol(SYMBOL::squarC_) },
            '(' => { self.loc.deepen(); self.key = symbol(SYMBOL::roundO_) },
            ')' => { self.loc.soften(); self.key = symbol(SYMBOL::roundC_) },
            '\\' => { self.key = symbol(SYMBOL::escape_) },
            '.' => { self.key = symbol(SYMBOL::dot_) },
            ',' => { self.key = symbol(SYMBOL::comma_) },
            ':' => { self.key = symbol(SYMBOL::colon_) },
            ';' => { self.key = symbol(SYMBOL::semi_) },
            '|' => { self.key = symbol(SYMBOL::pipe_) },
            '=' => { self.key = symbol(SYMBOL::equal_) },
            '>' => { self.key = symbol(SYMBOL::greater_) },
            '<' => { self.key = symbol(SYMBOL::less_) },
            '+' => { self.key = symbol(SYMBOL::plus_) },
            '-' => { self.key = symbol(SYMBOL::minus_) },
            '_' => { self.key = symbol(SYMBOL::under_) },
            '*' => { self.key = symbol(SYMBOL::star_) },
            '~' => { self.key = symbol(SYMBOL::home_) },
            '/' => { self.key = symbol(SYMBOL::root_) },
            '%' => { self.key = symbol(SYMBOL::percent_) },
            '^' => { self.key = symbol(SYMBOL::carret_) },
            '?' => { self.key = symbol(SYMBOL::query_) },
            '!' => { self.key = symbol(SYMBOL::bang_) },
            '&' => { self.key = symbol(SYMBOL::and_) },
            '@' => { self.key = symbol(SYMBOL::at_) },
            '#' => { self.key = symbol(SYMBOL::hash_) },
            '$' => { self.key = symbol(SYMBOL::dollar_) },
            '°' => { self.key = symbol(SYMBOL::degree_) },
            '§' => { self.key = symbol(SYMBOL::sign_) },
            _ => { self.key = illegal }
        }
    }
    fn alpha(&mut self, part: &mut parts::PART) {
        // println!("{} - {}", "enter", &part.curr_char());
        // println!("{} {} {}", &part.prev_char(), &part.curr_char(), &part.next_char());
        self.push_curr(part);
        while is_alpha(&part.next_char()) {
            part.bump(&mut self.loc);
            self.push_curr(part);
            // self.bump_next(part);
        }
        self.key = ident;
    }

    fn push_curr(&mut self, part: &mut parts::PART) {
        self.con.push_str(&part.curr_char().to_string());
    }

    fn bump_next(&mut self, part: &mut parts::PART) {
        part.bump(&mut self.loc);
        self.con.push_str(&part.curr_char().to_string());
    }

    fn bump_curr(&mut self, part: &mut parts::PART) {
        self.con.push_str(&part.curr_char().to_string());
        part.bump(&mut self.loc);
    }
}

