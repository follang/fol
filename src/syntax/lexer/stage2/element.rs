use std::fmt;
use colored::Colorize;
use crate::types::{Vod, Con, Win, SLIDER, error::*};
use crate::syntax::index;
use crate::syntax::point;
use crate::syntax::lexer::stage1;
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
            self.set_key(void(VOID::space_))
        } 
        // EOL => SEMICOLON
        else if matches!(el.curr()?.key(), KEYWORD::symbol(SYMBOL::semi_))
            && el.peek(0)?.key().is_void()
        {
            self.append(&el.peek(0)?.into());
            self.bump(el);
        }
        // EOL or SPACE => EOF
        else if ( matches!(el.curr()?.key(), KEYWORD::void(VOID::space_))
            || matches!(el.curr()?.key(), KEYWORD::void(VOID::endline_)) )
            && el.peek(0)?.key().is_eof()
        {
            self.append(&el.peek(0)?.into());
            self.set_key(void(VOID::endfile_));
            self.bump(el);
        }
        // operators
        else if el.curr()?.key().is_dot() && el.peek(0)?.key().is_dot() {
            self.append(&el.peek(0)?.into());
            self.bump(el);
            self.set_key(operator(OPERATOR::dd_));
            if el.peek(0)?.key().is_dot() {
                self.append(&el.peek(0)?.into());
                self.bump(el);
                self.set_key(operator(OPERATOR::ddd_));
            }
        }
        else if el.curr()?.key().is_symbol()
            && el.peek(0)?.key().is_symbol()
            && (!(matches!(el.curr()?.key(), KEYWORD::symbol(SYMBOL::semi_)))
            || !(matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::semi_))))
            // && (el.seek(0)?.key().is_void() || el.seek(0)?.key().is_bracket())
        {
            self.make_multi_operator(el)?;
        }
        else if matches!(el.curr()?.key(), KEYWORD::ident) 
        {
            self.set_key(ident)
        }
        // if self.key().is_eol() { self.set_key(symbol(SYMBOL::semi_)) }
        Ok(())
    }

    pub fn make_multi_operator(&mut self, el: &mut stage1::Elements) -> Vod {
        let mut content = String::new();
        content.push_str(el.curr()?.con());
        content.push_str(el.peek(0)?.con());
        match content.as_str() {
            ":=" => self.set_key(operator(OPERATOR::assign2_)),
            "::" => self.set_key(operator(OPERATOR::path_)),
            "=>" => self.set_key(operator(OPERATOR::flow_)),
            "->" => self.set_key(operator(OPERATOR::flow2_)),
            "==" => self.set_key(operator(OPERATOR::equal_)),
            "!=" => self.set_key(operator(OPERATOR::noteq_)),
            ">=" => self.set_key(operator(OPERATOR::greatereq_)),
            "<=" => self.set_key(operator(OPERATOR::lesseq_)),
            "+=" => self.set_key(operator(OPERATOR::addeq_)),
            "-=" => self.set_key(operator(OPERATOR::subtracteq_)),
            "*=" => self.set_key(operator(OPERATOR::multiplyeq_)),
            "/=" => self.set_key(operator(OPERATOR::divideeq_)),
            "<<" => self.set_key(operator(OPERATOR::lesser_)),
            ">>" => self.set_key(operator(OPERATOR::greater_)),
            _ => return Ok(()),
        }
        self.append(&el.peek(0)?.into());
        self.bump(el);
        Ok(())
    }

    pub fn make_comment(&mut self, el: &mut stage1::Elements) -> Vod {
        if matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::root_)) {
            while !el.peek(0)?.key().is_eol() {
                self.append(&el.peek(0)?.into());
                self.bump(el);
            }
        } else if matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::star_)) {
            while !(matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::star_))
                && matches!(el.peek(1)?.key(), KEYWORD::symbol(SYMBOL::root_)))
                || el.peek(0)?.key().is_eof()
            {
                self.append(&el.peek(0)?.into());
                self.bump(el);
            }
            self.append(&el.peek(0)?.into());
            self.bump(el);
        };
        self.set_key(comment);
        Ok(())
    }
    pub fn bump(&mut self, el: &mut stage1::Elements) {
        el.bump();
    }
}
