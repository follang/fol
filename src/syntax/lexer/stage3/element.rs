use std::fmt;
use colored::Colorize;
use crate::types::*;
use crate::syntax::index::*;
use crate::syntax::point;
use crate::syntax::lexer::stage2;
use crate::syntax::token::{
    literal::LITERAL,
    void::VOID,
    symbol::SYMBOL,
    operator::OPERATOR,
    buildin::BUILDIN,
};
use crate::syntax::token::{KEYWORD, KEYWORD::*};


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Element {
    key: KEYWORD,
    loc: point::Location,
    con: String,
}

impl std::default::Default for Element {
    fn default() -> Self {
        Self {
            key: KEYWORD::void(VOID::endfile_),
            loc: point::Location::default(),
            con: String::new(),
        }
    }
}

impl From<stage2::Element> for Element {
    fn from(stg1: stage2::Element) -> Self {
        Self { key: stg1.key().clone(), loc: stg1.loc().clone(), con: stg1.con().clone() }
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let con = if self.key().is_literal()
            || self.key().is_comment()
            || self.key().is_ident() { " ".to_string() + &self.con + " " } else { "".to_string() };
        write!(f, "{}\t{}{}", self.loc, self.key, con.black().on_red())
    }
}


impl Element {
    pub fn init(key: KEYWORD, loc: point::Location, con: String) -> Self { Self{ key, loc, con } }
    pub fn key(&self) -> KEYWORD { self.key.clone() }
    pub fn set_key(&mut self, k: KEYWORD) { self.key = k; }
    pub fn loc(&self) -> &point::Location { &self.loc }
    pub fn set_loc(&mut self, l: point::Location) { self.loc = l; }
    pub fn con(&self) -> &String { &self.con }
    pub fn set_con(&mut self, c: String) { self.con = c; }

    pub fn append(&mut self, other: &Element) {
        self.con.push_str(&other.con);
        self.loc.longer(&other.loc.len())
    }

    pub fn analyze(&mut self, el: &mut stage2::Elements) -> Vod {
        if el.curr(false)?.key().is_number() && el.seek(0, false)?.key().is_operator() {
            self.make_number(el)?;
        }else if el.curr(false)?.key().is_number() && !el.seek(0, false)?.key().is_void() {
            self.set_key(ident);
        }
        else if (el.curr(false)?.key().is_numberish() && el.seek(0, false)?.key().is_continue()) && 
            (el.peek(0, false)?.key().is_number() 
                && (el.seek(0, false)?.key().is_continue()  || el.seek(0, false)?.key().is_operator() || el.seek(0, false)?.key().is_eol()))
        {
            self.make_number(el)?;
        }
        Ok(())
    }
    pub fn make_number(&mut self, el: &mut stage2::Elements) -> Vod{
        self.set_key(literal(LITERAL::decimal_));

        // if el.curr(false)?.key().is_dot() && el.peek(0, false)?.key().is_symbol() {
            // self.make_multi_operator(el)?;
        // } else if el.curr(false)?.key().is_decimal() && el.peek(0, false)?.key().is_dot() && el.peek(1, false)?.key().is_symbol() {
            // return Ok(());
        // }

        if matches!(el.curr(false)?.key(), KEYWORD::symbol(SYMBOL::minus_)) && el.peek(0, false)?.key().is_decimal()
        {
            self.append(&el.peek(0, false)?.into());
            self.bump(el);
            if !el.peek(0, false)?.key().is_dot() || !el.peek(0, false)?.key().is_decimal() { return Ok(()) }
        }

        if el.curr(false)?.key().is_decimal() && el.peek(0, false)?.key().is_dot() && el.peek(1, false)?.key().is_decimal() {
            self.set_key(literal(LITERAL::float_));
            self.append(&el.peek(0, false)?.into());
            self.bump(el);
        }

        if el.curr(false)?.key().is_dot() && el.peek(0, false)?.key().is_decimal() {
            self.set_key(literal(LITERAL::float_));
            self.append(&el.peek(0, false)?.into());
            self.bump(el);
        }

        if el.peek(0, false)?.key().is_dot() && !(el.peek(1, false)?.key().is_ident() || el.peek(1, false)?.key().is_buildin()) {
            let mut elem = el.peek(0, false)?;
            elem.append(&el.peek(1, false)?);
            self.bump(el);
            return Err(catch!(Typo::LexerSpaceAdd{ 
                msg: Some(format!("Expected {} but {} was given", KEYWORD::void(VOID::space_), elem.key())),
                loc: Some(elem.loc().clone()),
                src: el.curr(false)?.loc().source(),
            }))
        }
        Ok(())
    }
    pub fn make_buildin(&mut self, el: &mut stage2::Elements) -> Vod {
        Ok(())
    }
    pub fn bump(&mut self, el: &mut stage2::Elements) {
        el.bump();
    }
}
