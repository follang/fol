#![allow(dead_code)]

use std::fmt;
use crate::syntax::point;
use crate::syntax::lexer::stage1;
use crate::types::*;
use crate::syntax::error::*;

use crate::syntax::token::KEYWORD::*;
use crate::syntax::token::*;


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
        write!(f, "{}\t{}  {}", self.loc, self.key, self.con)
    }
}


impl Element {
    pub fn init(key: KEYWORD, loc: point::Location, con: String) -> Self { Self{ key, loc, con } }
    pub fn key(&self) -> &KEYWORD { &self.key }
    pub fn set_key(&mut self, k: KEYWORD) { self.key = k; }
    pub fn loc(&self) -> &point::Location { &self.loc }
    pub fn con(&self) -> &String { &self.con }

    fn combine(&mut self, other: &Element) {
        self.con.push_str(&other.con);
        self.loc.longer(&other.loc.len())
    }

    pub fn analyze(&mut self, el: &mut stage1::Elements){
        // // EOL to SPACE
        if el.curr().key().is_eol()
            && (el.seek(0).key().is_nonterm()
                || el.peek(0).key().is_dot()
                || el.seek(0).key().is_operator())
        {
            self.set_key(void(VOID::space_))
        } else if matches!(el.curr().key(), KEYWORD::symbol(SYMBOL::semi_))
            && el.peek(0).key().is_void()
        {
            self.combine(&el.peek(0).into());
            el.bump();
        }
        // numbers
        else if matches!(el.curr().key(), KEYWORD::symbol(SYMBOL::dot_))
            && el.peek(0).key().is_number()
        {
            if el.seek(0).key().is_void() {
                self.make_number(el);
            }
        } else if (matches!(el.curr().key(), KEYWORD::symbol(SYMBOL::minus_))
            && el.peek(0).key().is_number())
            || el.curr().key().is_number()
        {
            if !el.seek(0).key().is_void()
                && matches!(el.curr().key(), KEYWORD::symbol(SYMBOL::minus_))
            {
                let key = el.seek(0).key().clone();
                //TODO: report error
            } else {
                self.make_number(el);
            }
        }
        // operators
        else if el.curr().key().is_symbol()
            && (matches!(el.curr().key(), KEYWORD::symbol(SYMBOL::semi_)))
            && (matches!(el.peek(0).key(), KEYWORD::symbol(SYMBOL::semi_)))
            && el.peek(0).key().is_symbol()
            && (el.seek(0).key().is_void() || el.seek(0).key().is_bracket())
        {
            self.make_multi_operator(el);
        }
        // options
        else if el.curr().key().is_symbol()
            && el.peek(0).key().is_assign()
            && (el.seek(0).key().is_terminal()
                || el.seek(0).key().is_eof()
                || el.seek(0).key().is_void())
        {
            self.make_syoption(el);
        }
        else if matches!(el.curr().key(), KEYWORD::ident(_)) {
            self.set_key(ident(Some(el.curr().con().to_string())))
        }
    }

    pub fn make_multi_operator(&mut self, el: &mut stage1::Elements) -> Self {
        let mut result = self.clone();
        while el.peek(0).key().is_symbol() && !el.peek(0).key().is_bracket() {
            result.combine(&el.peek(0).into());
            el.bump();
        }
        match result.con().as_str() {
            ":=" => result.set_key(operator(OPERATOR::assign2_)),
            "..." => result.set_key(operator(OPERATOR::ddd_)),
            ".." => result.set_key(operator(OPERATOR::dd_)),
            "=>" => result.set_key(operator(OPERATOR::flow_)),
            "->" => result.set_key(operator(OPERATOR::flow2_)),
            "==" => result.set_key(operator(OPERATOR::equal_)),
            "!=" => result.set_key(operator(OPERATOR::noteq_)),
            ">=" => result.set_key(operator(OPERATOR::greatereq_)),
            "<=" => result.set_key(operator(OPERATOR::lesseq_)),
            "+=" => result.set_key(operator(OPERATOR::addeq_)),
            "-=" => result.set_key(operator(OPERATOR::subtracteq_)),
            "*=" => result.set_key(operator(OPERATOR::multiplyeq_)),
            "/=" => result.set_key(operator(OPERATOR::divideeq_)),
            "<<" => result.set_key(operator(OPERATOR::shiftleft_)),
            ">>" => result.set_key(operator(OPERATOR::shiftright_)),
            _ => result.set_key(operator(OPERATOR::ANY)),
        }
        result
    }
    pub fn make_syoption(&mut self, el: &mut stage1::Elements) -> Self {
        let mut result = self.clone();
        match result.con().as_str() {
            "~" => result.set_key(option(OPTION::mut_)),
            "!" => result.set_key(option(OPTION::sta_)),
            "+" => result.set_key(option(OPTION::exp_)),
            "-" => result.set_key(option(OPTION::hid_)),
            "@" => result.set_key(option(OPTION::hep_)),
            _ => {}
        }
        result
    }

    pub fn make_number(&mut self, el: &mut stage1::Elements){
        if el.curr().key().is_dot() && el.peek(0).key().is_decimal() {
            self.set_key(literal(LITERAL::float_));
            self.combine(&el.peek(0).into());
            el.bump();
            if el.peek(0).key().is_dot()
                && el.peek(1).key().is_eol()
                && el.peek(2).key().is_ident()
            {
                return
            } else if el.peek(0).key().is_dot() && !el.peek(1).key().is_ident() {
                el.bump();
                //TODO: report error
            }
        } else if el.seek(0).key().is_continue()
            && el.curr().key().is_decimal()
            && el.peek(0).key().is_dot()
            && !el.peek(1).key().is_ident()
        {
            self.set_key(literal(LITERAL::float_));
            self.combine(&el.peek(0).into());
            el.bump();
            if el.peek(0).key().is_number() {
                self.combine(&el.peek(0).into());
                el.bump();
                if el.peek(0).key().is_dot() && el.peek(1).key().is_number() {
                    el.bump();
                    //TODO: report error
                }
            } else if !el.peek(0).key().is_void() {
                el.bump();
                //TODO: report error
            }
        };
    }
    pub fn make_comment(&mut self, el: &mut stage1::Elements) {
        if matches!(el.peek(0).key(), KEYWORD::symbol(SYMBOL::root_)) {
            while !el.peek(0).key().is_eol() {
                self.combine(&el.peek(0).into());
                el.bump();
            }
        } else if matches!(el.peek(0).key(), KEYWORD::symbol(SYMBOL::star_)) {
            while !(matches!(el.peek(0).key(), KEYWORD::symbol(SYMBOL::star_))
                && matches!(el.peek(1).key(), KEYWORD::symbol(SYMBOL::root_)))
                || el.peek(0).key().is_eof()
            {
                self.combine(&el.peek(0).into());
                el.bump();
            }
            self.combine(&el.peek(0).into());
            el.bump();
        };
        self.set_key(comment);
    }
}

pub struct Elements {
    elem: Box<dyn Iterator<Item = Element>>,
    win: Win<Element>,
    _in_count: usize,
}


impl Elements {
    pub fn init(dir: String) -> Self {
        let mut prev = Vec::with_capacity(SLIDER);
        let mut next = Vec::with_capacity(SLIDER);
        let mut elem = Box::new(elements(dir));
        for _ in 0..SLIDER { prev.push(Element::default()) }
        for _ in 0..SLIDER { next.push(elem.next().unwrap()) }
        Self {
            elem,
            win: (prev, Element::default(), next),
            _in_count: SLIDER
        }
    }
    pub fn curr(&self) -> Element {
        self.win.1.clone()
    }
    pub fn next_vec(&self) -> Vec<Element> {
        self.win.2.clone()
    }
    pub fn peek(&self, index: usize, ignore: bool) -> Element { 
        let mut u = if index > SLIDER { SLIDER } else { index };
        if ignore && self.next_vec()[u].key().is_space() && u < SLIDER { u += 1 };
        self.next_vec()[u].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Element> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn seek(&self, index: usize, ignore: bool) -> Element { 
        let mut u = if index > SLIDER { SLIDER } else { index };
        if ignore && self.next_vec()[u].key().is_space() && u < SLIDER { u += 1 };
        self.prev_vec()[u].clone() 
    }

    pub fn bump(&mut self) -> Opt<Element> {
        match self.elem.next() {
            Some(v) => {
                self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                self.win.1 = self.win.2[0].clone();
                self.win.2.remove(0); self.win.2.push(v);
                return Some(self.win.1.clone())
            },
            None => {
                if self._in_count > 0 {
                    self.win.0.remove(0); self.win.0.push(self.win.1.clone());
                    self.win.1 = self.win.2[0].clone();
                    self.win.2.remove(0); self.win.2.push(Element::default());
                    self._in_count -= 1;
                    return Some(self.win.1.clone())
                } else { return None }
            }
        }
    }
}

impl Iterator for Elements {
    type Item = Element;
    fn next(&mut self) -> Option<Element> {
        match self.bump() {
            Some(v) => Some(v),
            None => None
        }
    }
}

/// Creates a iterator that produces tokens from the input string.
pub fn elements(dir: String) -> impl Iterator<Item = Element>  {
    let mut stg = Box::new(stage1::Elements::init(dir));
    std::iter::from_fn(move || {
        if let Some(v) = stg.next() {
            let mut result: Element = v.into();
            result.analyze(&mut stg);
            return Some(result);
        }
        None
    })
}

