use std::fmt;
use colored::Colorize;
use crate::types::Vod;
use crate::syntax::point;
use crate::syntax::lexer::stage1;
use crate::syntax::token::{
    void::VOID,
    symbol::SYMBOL,
    operator::OPERATOR,
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
            key: KEYWORD::Void(VOID::EndFile),
            loc: point::Location::default(),
            con: String::new(),
        }
    }
}

impl From<stage1::Element> for Element {
    fn from(stg1: stage1::Element) -> Self {
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

    pub fn analyze(&mut self, el: &mut stage1::Elements) -> Vod {
        // EOL => SPACE
        if el.curr()?.key().is_eol()
            && (el.seek(0)?.key().is_nonterm()
                || el.peek(0)?.key().is_dot()
                || el.seek(0)?.key().is_operator())
        {
            self.set_key(Void(VOID::Space))
        } 
        // EOL => SEMICOLON
        else if matches!(el.curr()?.key(), KEYWORD::Symbol(SYMBOL::Semi))
            && el.peek(0)?.key().is_void()
        {
            self.append(&el.peek(0)?.into());
            self.bump(el);
        }
        // EOL or SPACE => EOF
        else if ( matches!(el.curr()?.key(), KEYWORD::Void(VOID::Space))
            || matches!(el.curr()?.key(), KEYWORD::Void(VOID::EndLine)) )
            && el.peek(0)?.key().is_eof()
        {
            self.append(&el.peek(0)?.into());
            self.set_key(Void(VOID::EndFile));
            self.bump(el);
        }
        // operators
        else if el.curr()?.key().is_dot() && el.peek(0)?.key().is_dot() {
            self.append(&el.peek(0)?.into());
            self.bump(el);
            self.set_key(Operator(OPERATOR::Dotdot));
            if el.peek(0)?.key().is_dot() {
                self.append(&el.peek(0)?.into());
                self.bump(el);
                self.set_key(Operator(OPERATOR::Dotdotdot));
            }
        }
        else if el.curr()?.key().is_symbol()
            && el.peek(0)?.key().is_symbol()
            && (!(matches!(el.curr()?.key(), KEYWORD::Symbol(SYMBOL::Semi)))
            || !(matches!(el.peek(0)?.key(), KEYWORD::Symbol(SYMBOL::Semi))))
            // && (el.seek(0)?.key().is_void() || el.seek(0)?.key().is_bracket())
        {
            self.make_multi_operator(el)?;
        }
        else if matches!(el.curr()?.key(), KEYWORD::Identifier) 
        {
            self.set_key(Identifier)
        }
        // if self.key().is_eol() { self.set_key(symbol(SYMBOL::semi_)) }
        Ok(())
    }

    pub fn make_multi_operator(&mut self, el: &mut stage1::Elements) -> Vod {
        let mut content = String::new();
        content.push_str(el.curr()?.con());
        content.push_str(el.peek(0)?.con());
        match content.as_str() {
            ":=" => self.set_key(Operator(OPERATOR::Assign)),
            "::" => self.set_key(Operator(OPERATOR::Path)),
            "=>" => self.set_key(Operator(OPERATOR::Flow)),
            "->" => self.set_key(Operator(OPERATOR::Flow2)),
            "==" => self.set_key(Operator(OPERATOR::Equal)),
            "!=" => self.set_key(Operator(OPERATOR::Noteq)),
            ">=" => self.set_key(Operator(OPERATOR::Greateq)),
            "<=" => self.set_key(Operator(OPERATOR::Lesseq)),
            "+=" => self.set_key(Operator(OPERATOR::Addeq)),
            "-=" => self.set_key(Operator(OPERATOR::Subeq)),
            "*=" => self.set_key(Operator(OPERATOR::Multeq)),
            "/=" => self.set_key(Operator(OPERATOR::Diveq)),
            "<<" => self.set_key(Operator(OPERATOR::Lesser)),
            ">>" => self.set_key(Operator(OPERATOR::Greater)),
            _ => return Ok(()),
        }
        self.append(&el.peek(0)?.into());
        self.bump(el);
        Ok(())
    }

    pub fn make_comment(&mut self, el: &mut stage1::Elements) -> Vod {
        if matches!(el.peek(0)?.key(), KEYWORD::Symbol(SYMBOL::Root)) {
            while !el.peek(0)?.key().is_eol() {
                self.append(&el.peek(0)?.into());
                self.bump(el);
            }
        } else if matches!(el.peek(0)?.key(), KEYWORD::Symbol(SYMBOL::Star)) {
            while !(matches!(el.peek(0)?.key(), KEYWORD::Symbol(SYMBOL::Star))
                && matches!(el.peek(1)?.key(), KEYWORD::Symbol(SYMBOL::Root)))
                || el.peek(0)?.key().is_eof()
            {
                self.append(&el.peek(0)?.into());
                self.bump(el);
            }
            self.append(&el.peek(0)?.into());
            self.bump(el);
        };
        self.set_key(Comment);
        Ok(())
    }
    pub fn bump(&mut self, el: &mut stage1::Elements) {
        el.bump();
    }
}
