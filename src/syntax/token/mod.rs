#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(unused_variables)]


use colored::Colorize;
use std::collections::HashMap;
use std::fmt;

pub mod literal;
pub mod void;
pub mod symbol;
pub mod operator;
pub mod buildin;
pub mod assign;
pub mod types;
pub mod option;
pub mod form;

use crate::syntax::token::{
    literal::LITERAL,
    void::VOID,
    symbol::SYMBOL,
    operator::OPERATOR,
    buildin::BUILDIN,
    assign::ASSIGN,
    types::TYPE,
    option::OPTION,
    form::FORM };

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KEYWORD {
    assign(ASSIGN),
    option(OPTION),
    types(TYPE),
    form(FORM),
    literal(LITERAL),
    buildin(BUILDIN),
    symbol(SYMBOL),
    operator(OPERATOR),
    void(VOID),
    ident(Option<String>),
    comment(Option<String>),
    illegal,
}

impl KEYWORD {
    pub fn is_assign(&self) -> bool {
        match *self {
            KEYWORD::assign(_) => true,
            _ => false,
        }
    }
    pub fn is_option(&self) -> bool {
        match *self {
            KEYWORD::option(_) => true,
            _ => false,
        }
    }
    pub fn is_ident(&self) -> bool {
        match *self {
            KEYWORD::ident(_) => true,
            _ => false,
        }
    }
    pub fn is_type(&self) -> bool {
        match *self {
            KEYWORD::types(_) => true,
            _ => false,
        }
    }
    pub fn is_form(&self) -> bool {
        match *self {
            KEYWORD::form(_) => true,
            _ => false,
        }
    }
    pub fn is_literal(&self) -> bool {
        match *self {
            KEYWORD::literal(_) => true,
            _ => false,
        }
    }
    pub fn is_buildin(&self) -> bool {
        match *self {
            KEYWORD::buildin(_) => true,
            _ => false,
        }
    }
    pub fn is_comment(&self) -> bool {
        match *self {
            KEYWORD::comment(_) => true,
            _ => false,
        }
    }
    pub fn is_open_bracket(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyO_) => true,
            KEYWORD::symbol(SYMBOL::squarO_) => true,
            KEYWORD::symbol(SYMBOL::roundO_) => true,
            _ => false,
        }
    }
    pub fn is_close_bracket(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyC_) => true,
            KEYWORD::symbol(SYMBOL::squarC_) => true,
            KEYWORD::symbol(SYMBOL::roundC_) => true,
            _ => false,
        }
    }
    pub fn is_bracket(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyC_) => true,
            KEYWORD::symbol(SYMBOL::squarC_) => true,
            KEYWORD::symbol(SYMBOL::roundC_) => true,
            KEYWORD::symbol(SYMBOL::curlyO_) => true,
            KEYWORD::symbol(SYMBOL::squarO_) => true,
            KEYWORD::symbol(SYMBOL::roundO_) => true,
            _ => false,
        }
    }
    pub fn is_decimal(&self) -> bool {
        match *self {
            KEYWORD::literal(LITERAL::decimal_) => true,
            KEYWORD::literal(LITERAL::float_) => true,
            _ => false,
        }
    }
    pub fn is_number(&self) -> bool {
        match *self {
            KEYWORD::literal(LITERAL::decimal_) => true,
            KEYWORD::literal(LITERAL::float_) => true,
            KEYWORD::literal(LITERAL::hexal_) => true,
            KEYWORD::literal(LITERAL::octal_) => true,
            KEYWORD::literal(LITERAL::binary_) => true,
            _ => false,
        }
    }
    pub fn is_symbol(&self) -> bool {
        if self.is_bracket() {
            false
        } else {
            match *self {
                KEYWORD::symbol(_) => true,
                _ => false,
            }
        }
    }
    pub fn is_operator(&self) -> bool {
        match *self {
            KEYWORD::operator(_) => true,
            _ => false,
        }
    }
    pub fn is_void(&self) -> bool {
        match *self {
            KEYWORD::void(_) => true,
            _ => false,
        }
    }
    pub fn is_eof(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endfile_) => true,
            _ => false,
        }
    }
    pub fn is_space(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::space_) => true,
            _ => false,
        }
    }
    pub fn is_eol(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endline_) => true,
            _ => false,
        }
    }
    pub fn is_nonterm(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::curlyO_) => true,
            KEYWORD::symbol(SYMBOL::squarO_) => true,
            KEYWORD::symbol(SYMBOL::roundO_) => true,
            KEYWORD::symbol(SYMBOL::dot_) => true,
            KEYWORD::symbol(SYMBOL::comma_) => true,
            _ => false,
        }
    }
    pub fn is_terminal(&self) -> bool {
        match *self {
            KEYWORD::void(VOID::endline_) => true,
            KEYWORD::symbol(SYMBOL::semi_) => true,
            _ => false,
        }
    }
    pub fn is_dot(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::dot_) => true,
            _ => false,
        }
    }
    pub fn is_comma(&self) -> bool {
        match *self {
            KEYWORD::symbol(SYMBOL::comma_) => true,
            _ => false,
        }
    }
    pub fn is_continue(&self) -> bool {
        if self.is_void() || self.is_bracket() || self.is_terminal() || self.is_comma() {
            true
        } else {
            false
        }
    }
}

impl fmt::Display for KEYWORD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KEYWORD::literal(v) => write!(f, "{}", v),
            KEYWORD::void(v) => write!(f, "{}", v),
            KEYWORD::symbol(v) => write!(f, "{}", v),
            KEYWORD::operator(v) => write!(f, "{}", v),
            KEYWORD::assign(v) => write!(f, "{}", v),
            KEYWORD::types(v) => write!(f, "{}", v),
            KEYWORD::buildin(v) => write!(f, "{}", v),
            KEYWORD::form(v) => write!(f, "{}", v),
            KEYWORD::option(v) => write!(f, "{}", v),
            KEYWORD::ident(Some(v)) => write!(f, "{}: {}", " IDENT    ".black().on_red(), v.to_string()),
            KEYWORD::ident(None) => write!(f, "{}", " IDENT    ".black().on_red()),
            KEYWORD::comment(Some(v)) => write!(f, "{}: {}", " COMMENT  ".black().on_red(), v.to_string()),
            KEYWORD::comment(None) => write!(f, "{}", " COMMENT  ".black().on_red()),
            KEYWORD::illegal => write!(f, "{}", " ILLEGAL  ".black().on_red()),
        }
    }
}

pub fn get_keyword() -> HashMap<String, KEYWORD> {
    let mut keywords: HashMap<String, KEYWORD> = HashMap::new();
    keywords.insert(String::from("use"), KEYWORD::assign(ASSIGN::use_));
    keywords
}
pub mod part {
    pub fn is_eof(ch: &char) -> bool {
        return *ch == '\0';
    }

    pub fn is_eol(ch: &char) -> bool {
        return *ch == '\n' || *ch == '\r';
    }

    pub fn is_space(ch: &char) -> bool {
        return *ch == ' ' || *ch == '\t';
    }

    pub fn is_digit(ch: &char) -> bool {
        return '0' <= *ch && *ch <= '9';
    }

    pub fn is_alpha(ch: &char) -> bool {
        return 'a' <= *ch && *ch <= 'z' || 'A' <= *ch && *ch <= 'Z' || *ch == '_';
    }

    pub fn is_bracket(ch: &char) -> bool {
        return *ch == '{' || *ch == '[' || *ch == '(' || *ch == ')' || *ch == ']' || *ch == '}';
    }

    pub fn is_symbol(ch: &char) -> bool {
        return '!' <= *ch && *ch <= '/'
            || ':' <= *ch && *ch <= '@'
            || '[' <= *ch && *ch <= '^'
            || '{' <= *ch && *ch <= '~';
    }

    pub fn is_oct_digit(ch: &char) -> bool {
        return '0' <= *ch && *ch <= '7' || *ch == '_';
    }
    pub fn is_hex_digit(ch: &char) -> bool {
        return '0' <= *ch && *ch <= '9'
            || 'a' <= *ch && *ch <= 'f'
            || 'A' <= *ch && *ch <= 'F'
            || *ch == '_';
    }

    pub fn is_alphanumeric(ch: &char) -> bool {
        return is_digit(ch) || is_alpha(ch);
    }

    pub fn is_void(ch: &char) -> bool {
        return is_eol(ch) || is_space(ch);
    }
}
