use colored::Colorize;
use std::collections::HashMap;
use std::fmt;

pub mod help;

pub mod literal;
pub mod void;
pub mod symbol;
pub mod operator;
pub mod buildin;


pub use crate::syntax::token::{
    literal::LITERAL,
    void::VOID,
    symbol::SYMBOL,
    operator::OPERATOR,
    buildin::BUILDIN,
};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum KEYWORD {
    Literal(LITERAL),
    Keyword(BUILDIN),
    Symbol(SYMBOL),
    Operator(OPERATOR),
    Void(VOID),
    Identifier,
    Comment,
    Illegal,
}

impl KEYWORD {
    pub fn is_assign(&self) -> bool {
        match *self {
            KEYWORD::Keyword(BUILDIN::Use) => true,
            KEYWORD::Keyword(BUILDIN::Def) => true,
            KEYWORD::Keyword(BUILDIN::Var) => true,
            KEYWORD::Keyword(BUILDIN::Fun) => true,
            KEYWORD::Keyword(BUILDIN::Pro) => true,
            KEYWORD::Keyword(BUILDIN::Log) => true,
            KEYWORD::Keyword(BUILDIN::Typ) => true,
            KEYWORD::Keyword(BUILDIN::Ali) => true,
            KEYWORD::Keyword(BUILDIN::Imp) => true,
            KEYWORD::Keyword(BUILDIN::Lab) => true,
            KEYWORD::Keyword(BUILDIN::Con) => true,
            _ => false,
        }
    }

    pub fn is_ident(&self) -> bool {
        match *self {
            KEYWORD::Identifier => true,
            _ => false,
        }
    }
    pub fn is_literal(&self) -> bool {
        match *self {
            KEYWORD::Literal(_) => true,
            _ => false,
        }
    }
    pub fn is_buildin(&self) -> bool {
        match *self {
            KEYWORD::Keyword(_) => true,
            _ => false,
        }
    }
    pub fn is_illegal(&self) -> bool {
        match *self {
            KEYWORD::Illegal => true,
            _ => false,
        }
    }
    pub fn is_comment(&self) -> bool {
        match *self {
            KEYWORD::Comment => true,
            _ => false,
        }
    }
    pub fn is_open_bracket(&self) -> bool {
        match *self {
            KEYWORD::Symbol(SYMBOL::CurlyO) => true,
            KEYWORD::Symbol(SYMBOL::SquarO) => true,
            KEYWORD::Symbol(SYMBOL::RoundO) => true,
            _ => false,
        }
    }
    pub fn is_close_bracket(&self) -> bool {
        match *self {
            KEYWORD::Symbol(SYMBOL::CurlyC) => true,
            KEYWORD::Symbol(SYMBOL::SquarC) => true,
            KEYWORD::Symbol(SYMBOL::RoundC) => true,
            _ => false,
        }
    }
    pub fn is_bracket(&self) -> bool {
        if self.is_open_bracket() || self.is_close_bracket() {
            true
        } else { false }
    }
    pub fn is_decimal(&self) -> bool {
        match *self {
            KEYWORD::Literal(LITERAL::Deciaml) => true,
            KEYWORD::Literal(LITERAL::Float) => true,
            _ => false,
        }
    }
    pub fn is_number(&self) -> bool {
        match *self {
            KEYWORD::Literal(LITERAL::Deciaml) => true,
            KEYWORD::Literal(LITERAL::Float) => true,
            KEYWORD::Literal(LITERAL::Hexal) => true,
            KEYWORD::Literal(LITERAL::Octal) => true,
            KEYWORD::Literal(LITERAL::Binary) => true,
            _ => false,
        }
    }
    pub fn is_numberish(&self) -> bool {
        if self.is_number()
            || matches!(self, KEYWORD::Symbol(SYMBOL::Dot))
            || matches!(self, KEYWORD::Symbol(SYMBOL::Minus)) 
        { true } else { false }
    }
    pub fn is_symbol(&self) -> bool {
        if self.is_bracket() {
            false
        } else {
            match *self {
                KEYWORD::Symbol(_) => true,
                _ => false,
            }
        }
    }
    pub fn is_operator(&self) -> bool {
        match *self {
            KEYWORD::Operator(_) => true,
            _ => false,
        }
    }
    pub fn is_void(&self) -> bool {
        match *self {
            KEYWORD::Void(_) => true,
            KEYWORD::Illegal => true,
            _ => false,
        }
    }
    pub fn is_eof(&self) -> bool {
        match *self {
            KEYWORD::Void(VOID::EndFile) => true,
            _ => false,
        }
    }
    pub fn is_space(&self) -> bool {
        match *self {
            KEYWORD::Void(VOID::Space) => true,
            _ => false,
        }
    }
    pub fn is_eol(&self) -> bool {
        match *self {
            KEYWORD::Void(VOID::EndLine) => true,
            _ => false,
        }
    }
    pub fn is_nonterm(&self) -> bool {
        match *self {
            KEYWORD::Symbol(SYMBOL::CurlyO) => true,
            KEYWORD::Symbol(SYMBOL::SquarO) => true,
            KEYWORD::Symbol(SYMBOL::RoundO) => true,
            KEYWORD::Symbol(SYMBOL::Dot) => true,
            KEYWORD::Symbol(SYMBOL::Comma) => true,
            _ => false,
        }
    }
    pub fn is_terminal(&self) -> bool {
        match *self {
            KEYWORD::Void(VOID::EndLine) => true,
            KEYWORD::Void(VOID::EndFile) => true,
            KEYWORD::Symbol(SYMBOL::Semi) => true,
            _ => false,
        }
    }
    pub fn is_dot(&self) -> bool {
        match *self {
            KEYWORD::Symbol(SYMBOL::Dot) => true,
            _ => false,
        }
    }
    pub fn is_comma(&self) -> bool {
        match *self {
            KEYWORD::Symbol(SYMBOL::Comma) => true,
            _ => false,
        }
    }
    pub fn is_continue(&self) -> bool {
        if self.is_void() 
            || self.is_bracket()
            || self.is_terminal()
            || self.is_comma() {
                true
        } else {
            false
        }
    }
}

impl fmt::Display for KEYWORD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KEYWORD::Literal(v) => write!(f, "{}", v),
            KEYWORD::Void(v) => write!(f, "{}", v),
            KEYWORD::Symbol(v) => write!(f, "{}", v),
            KEYWORD::Operator(v) => write!(f, "{}", v),
            // KEYWORD::assign(v) => write!(f, "{}", v),
            KEYWORD::Keyword(v) => write!(f, "{}", v),
            KEYWORD::Identifier => write!(f, "{}", " IDENT    ".black().on_red()),
            KEYWORD::Comment => write!(f, "{}", " COMMENT  ".black().on_red()),
            KEYWORD::Illegal => write!(f, "{}", " ILLEGAL  ".black().on_green()),
        }
    }
}

pub fn get_keyword() -> HashMap<String, KEYWORD> {
    let mut keywords: HashMap<String, KEYWORD> = HashMap::new();
    keywords.insert(String::from("use"), KEYWORD::Keyword(BUILDIN::Use));
    keywords
}
