use std::fmt;
use colored::Colorize;
use crate::types::*;
use crate::syntax::index::*;
use crate::syntax::point;
use crate::syntax::lexer::stage1;
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
            key: KEYWORD::illegal,
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
            || self.key().is_orbit()
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

    fn combine(&mut self, other: &Element) {
        self.con.push_str(&other.con);
        self.loc.longer(&other.loc.len())
    }

    pub fn analyze(&mut self, el: &mut stage1::Elements, src: &source::Source) -> Vod {
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
            self.combine(&el.peek(0)?.into());
            self.bump(el);
        }
        // EOL or SPACE => EOF
        else if ( matches!(el.curr()?.key(), KEYWORD::void(VOID::space_))
            || matches!(el.curr()?.key(), KEYWORD::void(VOID::endline_)) )
            && el.peek(0)?.key().is_eof()
        {
            self.combine(&el.peek(0)?.into());
            self.set_key(void(VOID::endfile_));
            self.bump(el);
        }
        // numberfile_
        else if matches!(el.curr()?.key(), KEYWORD::symbol(SYMBOL::dot_))
            && el.peek(0)?.key().is_number()
        {
            if el.seek(0)?.key().is_void() {
                self.make_number(el, src)?;
            }
        } else if (matches!(el.curr()?.key(), KEYWORD::symbol(SYMBOL::minus_))
            && el.peek(0)?.key().is_number())
            || el.curr()?.key().is_number()
        {
            if !el.seek(0)?.key().is_void()
                && matches!(el.curr()?.key(), KEYWORD::symbol(SYMBOL::minus_))
            {
                let key = el.seek(0)?.key().clone();
                //TODO: report error
            } else {
                self.make_number(el, src)?;
            }
        }
        // operators
        else if el.curr()?.key().is_symbol()
            && el.peek(0)?.key().is_symbol()
            && (!(matches!(el.curr()?.key(), KEYWORD::symbol(SYMBOL::semi_)))
            || !(matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::semi_))))
            && (el.seek(0)?.key().is_void() || el.seek(0)?.key().is_bracket())
        {
            self.make_multi_operator(el, src)?;
        }
        // options
        else if el.curr()?.key().is_symbol()
            && el.peek(0)?.key().is_assign()
            && (el.seek(0)?.key().is_terminal()
                || el.seek(0)?.key().is_illegal()
                || el.seek(0)?.key().is_eof()
                || el.seek(0)?.key().is_void())
        {
            self.make_syoption(el, src)?;
        }
        else if matches!(el.curr()?.key(), KEYWORD::ident) {
            self.set_key(ident)
        }
        Ok(())
    }

    pub fn make_multi_operator(&mut self, el: &mut stage1::Elements, src: &source::Source) -> Vod {
        while el.peek(0)?.key().is_symbol() && !el.peek(0)?.key().is_bracket() {
            self.combine(&el.peek(0)?.into());
            self.bump(el);
        }
        match self.con().as_str() {
            ":=" => self.set_key(operator(OPERATOR::assign2_)),
            "..." => self.set_key(operator(OPERATOR::ddd_)),
            ".." => self.set_key(operator(OPERATOR::dd_)),
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
            "<<" => self.set_key(operator(OPERATOR::shiftleft_)),
            ">>" => self.set_key(operator(OPERATOR::shiftright_)),
            _ => self.set_key(operator(OPERATOR::ANY)),
        }
        Ok(())
    }
    pub fn make_syoption(&mut self, el: &mut stage1::Elements, src: &source::Source) -> Vod {
        match self.con().as_str() {
            "~" => self.set_key(option(OPTION::mut_)),
            "!" => self.set_key(option(OPTION::sta_)),
            "+" => self.set_key(option(OPTION::exp_)),
            "-" => self.set_key(option(OPTION::hid_)),
            "@" => self.set_key(option(OPTION::hep_)),
            _ => {}
        }
        Ok(())
    }

    pub fn make_number(&mut self, el: &mut stage1::Elements, src: &source::Source) -> Vod{
        if el.curr()?.key().is_dot() && el.peek(0)?.key().is_decimal() {
            self.set_key(literal(LITERAL::float_));
            self.combine(&el.peek(0)?.into());
            self.bump(el);
            if el.peek(0)?.key().is_dot()
                && el.peek(1)?.key().is_eol()
                && el.peek(2)?.key().is_ident()
            {
                return Ok(())
            } else if el.peek(0)?.key().is_dot() && !el.peek(1)?.key().is_ident() {
                let elem = el.peek(0)?;
                return Err(catch!(Typo::LexerSpaceAdd{ 
                    msg: Some(format!("Expected {} but {} was given", KEYWORD::void(VOID::space_), elem.key())),
                    loc: Some(elem.loc().clone()),
                    src: src.clone(),
                }))
            }
        } else if el.seek(0)?.key().is_continue()
            && el.curr()?.key().is_decimal()
            && el.peek(0)?.key().is_dot()
            && !el.peek(1)?.key().is_ident()
        {
            self.set_key(literal(LITERAL::float_));
            self.combine(&el.peek(0)?.into());
            self.bump(el);
            if el.peek(0)?.key().is_number() {
                self.combine(&el.peek(0)?.into());
                self.bump(el);
                if el.peek(0)?.key().is_dot() && el.peek(1)?.key().is_number() {
                    self.bump(el);
                    //TODO: report error
                }
            } else if !el.peek(0)?.key().is_void() {
                self.bump(el);
                //TODO: report error
            }
        };
        Ok(())
    }
    pub fn make_comment(&mut self, el: &mut stage1::Elements, src: &source::Source) -> Vod {
        if matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::root_)) {
            while !el.peek(0)?.key().is_eol() {
                self.combine(&el.peek(0)?.into());
                self.bump(el);
            }
        } else if matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::star_)) {
            while !(matches!(el.peek(0)?.key(), KEYWORD::symbol(SYMBOL::star_))
                && matches!(el.peek(1)?.key(), KEYWORD::symbol(SYMBOL::root_)))
                || el.peek(0)?.key().is_eof()
            {
                self.combine(&el.peek(0)?.into());
                self.bump(el);
            }
            self.combine(&el.peek(0)?.into());
            self.bump(el);
        };
        self.set_key(comment);
        Ok(())
    }
    pub fn bump(&mut self, el: &mut stage1::Elements) {
        el.bump();
    }
}
