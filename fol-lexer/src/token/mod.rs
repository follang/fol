use colored::Colorize;
use std::collections::HashMap;
use std::fmt;

pub mod help;

pub mod buildin;
pub mod literal;
pub mod operator;
pub mod symbol;
pub mod void;

pub use crate::token::{
    buildin::BUILDIN, literal::LITERAL, operator::OPERATOR, symbol::SYMBOL, void::VOID,
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
        matches!(
            *self,
            KEYWORD::Keyword(BUILDIN::Use)
                | KEYWORD::Keyword(BUILDIN::Def)
                | KEYWORD::Keyword(BUILDIN::Seg)
                | KEYWORD::Keyword(BUILDIN::Var)
                | KEYWORD::Keyword(BUILDIN::Fun)
                | KEYWORD::Keyword(BUILDIN::Pro)
                | KEYWORD::Keyword(BUILDIN::Typ)
                | KEYWORD::Keyword(BUILDIN::Ali)
                | KEYWORD::Keyword(BUILDIN::Imp)
                | KEYWORD::Keyword(BUILDIN::Lab)
                | KEYWORD::Keyword(BUILDIN::Con)
        )
    }

    pub fn is_ident(&self) -> bool {
        matches!(*self, KEYWORD::Identifier)
    }
    pub fn is_literal(&self) -> bool {
        matches!(*self, KEYWORD::Literal(_))
    }
    pub fn is_buildin(&self) -> bool {
        matches!(*self, KEYWORD::Keyword(_))
    }
    pub fn is_illegal(&self) -> bool {
        matches!(*self, KEYWORD::Illegal)
    }
    pub fn is_comment(&self) -> bool {
        matches!(*self, KEYWORD::Comment)
    }
    pub fn is_open_bracket(&self) -> bool {
        matches!(
            *self,
            KEYWORD::Symbol(SYMBOL::CurlyO)
                | KEYWORD::Symbol(SYMBOL::SquarO)
                | KEYWORD::Symbol(SYMBOL::RoundO)
        )
    }
    pub fn is_close_bracket(&self) -> bool {
        matches!(
            *self,
            KEYWORD::Symbol(SYMBOL::CurlyC)
                | KEYWORD::Symbol(SYMBOL::SquarC)
                | KEYWORD::Symbol(SYMBOL::RoundC)
        )
    }
    pub fn is_bracket(&self) -> bool {
        self.is_open_bracket() || self.is_close_bracket()
    }
    pub fn is_decimal(&self) -> bool {
        matches!(
            *self,
            KEYWORD::Literal(LITERAL::Deciaml) | KEYWORD::Literal(LITERAL::Float)
        )
    }
    pub fn is_number(&self) -> bool {
        matches!(
            *self,
            KEYWORD::Literal(LITERAL::Deciaml)
                | KEYWORD::Literal(LITERAL::Float)
                | KEYWORD::Literal(LITERAL::Hexal)
                | KEYWORD::Literal(LITERAL::Octal)
                | KEYWORD::Literal(LITERAL::Binary)
        )
    }
    pub fn is_numberish(&self) -> bool {
        self.is_number()
            || matches!(self, KEYWORD::Symbol(SYMBOL::Dot))
            || matches!(self, KEYWORD::Symbol(SYMBOL::Minus))
    }
    pub fn is_symbol(&self) -> bool {
        if self.is_bracket() {
            false
        } else {
            matches!(*self, KEYWORD::Symbol(_))
        }
    }
    pub fn is_operator(&self) -> bool {
        matches!(*self, KEYWORD::Operator(_))
    }
    pub fn is_void(&self) -> bool {
        matches!(*self, KEYWORD::Void(_) | KEYWORD::Illegal)
    }
    pub fn is_eof(&self) -> bool {
        matches!(*self, KEYWORD::Void(VOID::EndFile))
    }
    pub fn is_space(&self) -> bool {
        matches!(*self, KEYWORD::Void(VOID::Space))
    }
    pub fn is_eol(&self) -> bool {
        matches!(*self, KEYWORD::Void(VOID::EndLine))
    }
    pub fn is_nonterm(&self) -> bool {
        matches!(
            *self,
            KEYWORD::Symbol(SYMBOL::CurlyO)
                | KEYWORD::Symbol(SYMBOL::SquarO)
                | KEYWORD::Symbol(SYMBOL::RoundO)
                | KEYWORD::Symbol(SYMBOL::Dot)
                | KEYWORD::Symbol(SYMBOL::Comma)
        )
    }
    pub fn is_terminal(&self) -> bool {
        matches!(
            *self,
            KEYWORD::Void(VOID::EndLine)
                | KEYWORD::Void(VOID::EndFile)
                | KEYWORD::Symbol(SYMBOL::Semi)
        )
    }
    pub fn is_dot(&self) -> bool {
        matches!(*self, KEYWORD::Symbol(SYMBOL::Dot))
    }
    pub fn is_comma(&self) -> bool {
        matches!(*self, KEYWORD::Symbol(SYMBOL::Comma))
    }
    pub fn is_continue(&self) -> bool {
        self.is_void() || self.is_bracket() || self.is_terminal() || self.is_comma()
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
