#![allow(dead_code)]

use std::fmt;
use crate::syntax::point;
use crate::syntax::scan::source;
// use crate::syntax::scan::text;
use crate::syntax::scan::text2;

use crate::syntax::token::KEYWORD::*;
use crate::syntax::token::*;
use crate::syntax::error::*;


const SLIDER: usize = 9;


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

impl Element {
    pub fn empty(key: KEYWORD, loc: point::Location, con: String) -> Self {
        Element { key, loc, con }
    }
    pub fn key(&self) -> &KEYWORD {
        &self.key
    }
    pub fn loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn con(&self) -> &String {
        &self.con
    }
    pub fn set_key(&mut self, k: KEYWORD) {
        self.key = k;
    }
}

impl Element {
   fn comment(&mut self, code: &mut text2::Text) {
       self.con.push_str(&code.curr_char().to_string());
       self.bump(code);
       if code.curr_char() == '/' {
           self.bump(code);
           while !is_eol(&code.next_char()) {
               if is_eof(&code.next_char()) { break };
               self.bump(code);
           }
       }
       if code.curr_char() == '*' {
           self.bump(code);
           while !(code.curr_char() == '*' && code.next_char() == '/') {
               if is_eof(&code.next_char()) { break };
               self.bump(code);
           }
           self.bump(code);
           //TODO: double check
           if is_space(&code.next_char()) {
               self.bump(code);
           }
       }
       self.key = comment;
   }
   fn endline(&mut self, code: &mut text2::Text, terminated: bool) {
       self.push(code);
       self.key = void(VOID::endline_);
       while is_eol(&code.next_char()) || is_space(&code.next_char()) {
           self.loc.new_line();
           self.bump(code);
       }
       self.con = " ".to_string();
   }
   fn space(&mut self, code: &mut text2::Text) {
       self.push(code);
       while is_space(&code.next_char()) {
           self.bump(code);
       }
       if is_eol(&code.next_char()) {
           self.bump(code);
           self.endline(code, false);
           return;
       }
       self.key = void(VOID::space_);
       self.con = " ".to_string();
   }
   fn digit(&mut self, code: &mut text2::Text) {
       if code.curr_char() == '0'
           && (code.next_char() == 'x' || code.next_char() == 'o' || code.next_char() == 'b')
       {
           self.push(code);
           if code.next_char() == 'x' {
               self.bump(code);
               self.key = literal(LITERAL::hexal_);
               while is_hex_digit(&code.next_char()) {
                   self.bump(code);
               }
           } else if code.next_char() == 'o' {
               self.bump(code);
               self.key = literal(LITERAL::octal_);
               while is_oct_digit(&code.next_char()) {
                   self.bump(code);
               }
           } else if code.next_char() == 'b' {
               self.bump(code);
               self.key = literal(LITERAL::binary_);
               while code.next_char() == '0' || code.next_char() == '1' || code.next_char() == '_'
               {
                   self.bump(code);
               }
           }
       } else {
           self.push(code);
           self.key = literal(LITERAL::decimal_);
           while is_digit(&code.next_char()) || code.next_char() == '_' {
               self.bump(code);
           }
       }
   }
   fn encap(&mut self, code: &mut text2::Text) {
       let litsym = code.curr_char();
       if litsym == '`' {
           self.key = comment;
       } else if litsym == '\'' {
           self.key = literal(LITERAL::char_);
       } else {
           self.key = literal(LITERAL::string_);
       }
       self.push(code);
       while code.next_char() != litsym || (code.next_char() == litsym && code.curr_char() == '\\')
       {
           if code.next_char() != litsym && code.next_char() == '\0' {
               self.key = illegal;
               break;
           }
           self.bump(code);
       }
       self.bump(code);
   }
   fn symbol(&mut self, code: &mut text2::Text) {
       self.push(code);
       self.key = symbol(SYMBOL::curlyC_);
       match code.curr_char() {
           '{' => {
               self.loc.deepen();
               self.key = symbol(SYMBOL::curlyO_)
           }
           '}' => {
               self.loc.soften();
               self.key = symbol(SYMBOL::curlyC_)
           }
           '[' => {
               self.loc.deepen();
               self.key = symbol(SYMBOL::squarO_)
           }
           ']' => {
               self.loc.soften();
               self.key = symbol(SYMBOL::squarC_)
           }
           '(' => {
               self.loc.deepen();
               self.key = symbol(SYMBOL::roundO_)
           }
           ')' => {
               self.loc.soften();
               self.key = symbol(SYMBOL::roundC_)
           }
           ';' => self.key = symbol(SYMBOL::semi_),
           '\\' => self.key = symbol(SYMBOL::escape_),
           '.' => self.key = symbol(SYMBOL::dot_),
           ',' => self.key = symbol(SYMBOL::comma_),
           ':' => self.key = symbol(SYMBOL::colon_),
           '|' => self.key = symbol(SYMBOL::pipe_),
           '=' => self.key = symbol(SYMBOL::equal_),
           '>' => self.key = symbol(SYMBOL::greater_),
           '<' => self.key = symbol(SYMBOL::less_),
           '+' => self.key = symbol(SYMBOL::plus_),
           '-' => self.key = symbol(SYMBOL::minus_),
           '_' => self.key = symbol(SYMBOL::under_),
           '*' => self.key = symbol(SYMBOL::star_),
           '~' => self.key = symbol(SYMBOL::home_),
           '/' => self.key = symbol(SYMBOL::root_),
           '%' => self.key = symbol(SYMBOL::percent_),
           '^' => self.key = symbol(SYMBOL::carret_),
           '?' => self.key = symbol(SYMBOL::query_),
           '!' => self.key = symbol(SYMBOL::bang_),
           '&' => self.key = symbol(SYMBOL::and_),
           '@' => self.key = symbol(SYMBOL::at_),
           '#' => self.key = symbol(SYMBOL::hash_),
           '$' => self.key = symbol(SYMBOL::dollar_),
           '°' => self.key = symbol(SYMBOL::degree_),
           '§' => self.key = symbol(SYMBOL::sign_),
           _ => self.key = illegal,
       }
   }
   fn alpha(&mut self, code: &mut text2::Text) {
       let mut con = code.curr_char().to_string();
       self.push(code);
       while is_alpha(&code.next_char()) || is_digit(&code.next_char()) {
           self.bump(code);
           con.push(code.curr_char().clone());
       }
       match self.con().as_str() {
           "use" => self.set_key(assign(ASSIGN::use_)),
           "def" => self.set_key(assign(ASSIGN::def_)),
           "var" => self.set_key(assign(ASSIGN::var_)),
           "fun" => self.set_key(assign(ASSIGN::fun_)),
           "pro" => self.set_key(assign(ASSIGN::pro_)),
           "log" => self.set_key(assign(ASSIGN::log_)),
           "typ" => self.set_key(assign(ASSIGN::typ_)),
           "ali" => self.set_key(assign(ASSIGN::ali_)),
           "int" => self.set_key(types(TYPE::int_)),
           "flt" => self.set_key(types(TYPE::flt_)),
           "chr" => self.set_key(types(TYPE::chr_)),
           "bol" => self.set_key(types(TYPE::bol_)),
           "arr" => self.set_key(types(TYPE::arr_)),
           "vec" => self.set_key(types(TYPE::vec_)),
           "seq" => self.set_key(types(TYPE::seq_)),
           "mat" => self.set_key(types(TYPE::mat_)),
           "set" => self.set_key(types(TYPE::set_)),
           "map" => self.set_key(types(TYPE::map_)),
           "axi" => self.set_key(types(TYPE::axi_)),
           "tab" => self.set_key(types(TYPE::tab_)),
           "str" => self.set_key(types(TYPE::str_)),
           "num" => self.set_key(types(TYPE::num_)),
           "ptr" => self.set_key(types(TYPE::ptr_)),
           "err" => self.set_key(types(TYPE::err_)),
           "opt" => self.set_key(types(TYPE::opt_)),
           "nev" => self.set_key(types(TYPE::nev_)),
           "uni" => self.set_key(types(TYPE::uni_)),
           "any" => self.set_key(types(TYPE::any_)),
           "non" => self.set_key(types(TYPE::non_)),
           "nil" => self.set_key(types(TYPE::nil_)),
           "rec" => self.set_key(types(TYPE::rec_)),
           "ent" => self.set_key(types(TYPE::ent_)),
           "blu" => self.set_key(types(TYPE::blu_)),
           "std" => self.set_key(types(TYPE::std_)),
           "loc" => self.set_key(types(TYPE::loc_)),
           "url" => self.set_key(types(TYPE::url_)),
           "blk" => self.set_key(types(TYPE::blk_)),
           "rut" => self.set_key(types(TYPE::rut_)),
           "pat" => self.set_key(types(TYPE::pat_)),
           "gen" => self.set_key(types(TYPE::gen_)),
           "not" => self.set_key(buildin(BUILDIN::not_)),
           "or" => self.set_key(buildin(BUILDIN::or_)),
           "xor" => self.set_key(buildin(BUILDIN::xor_)),
           "and" => self.set_key(buildin(BUILDIN::and_)),
           "nand" => self.set_key(buildin(BUILDIN::nand_)),
           "as" => self.set_key(buildin(BUILDIN::as_)),
           "if" => self.set_key(buildin(BUILDIN::if_)),
           "when" => self.set_key(buildin(BUILDIN::when_)),
           "loop" => self.set_key(buildin(BUILDIN::loop_)),
           "is" => self.set_key(buildin(BUILDIN::is_)),
           "has" => self.set_key(buildin(BUILDIN::has_)),
           "in" => self.set_key(buildin(BUILDIN::in_)),
           "case" => self.set_key(buildin(BUILDIN::case_)),
           "this" => self.set_key(buildin(BUILDIN::this_)),
           "self" => self.set_key(buildin(BUILDIN::self_)),
           "break" => self.set_key(buildin(BUILDIN::break_)),
           "return" => self.set_key(buildin(BUILDIN::return_)),
           "yeild" => self.set_key(buildin(BUILDIN::yeild_)),
           "panic" => self.set_key(buildin(BUILDIN::panic_)),
           "report" => self.set_key(buildin(BUILDIN::report_)),
           "check" => self.set_key(buildin(BUILDIN::check_)),
           "assert" => self.set_key(buildin(BUILDIN::assert_)),
           "where" => self.set_key(buildin(BUILDIN::where_)),
           "true" => self.set_key(buildin(BUILDIN::true_)),
           "false" => self.set_key(buildin(BUILDIN::false_)),
           "each" => self.set_key(buildin(BUILDIN::each_)),
           "for" => self.set_key(buildin(BUILDIN::for_)),
           "do" => self.set_key(buildin(BUILDIN::do_)),
           "go" => self.set_key(buildin(BUILDIN::go_)),
           "get" => self.set_key(buildin(BUILDIN::get_)),
           "let" => self.set_key(buildin(BUILDIN::let_)),
           "mut" => self.set_key(option(OPTION::mut_)),
           "imu" => self.set_key(option(OPTION::imu_)),
           "sta" => self.set_key(option(OPTION::sta_)),
           "exp" => self.set_key(option(OPTION::exp_)),
           "nor" => self.set_key(option(OPTION::nor_)),
           "hid" => self.set_key(option(OPTION::hid_)),
           "stk" => self.set_key(option(OPTION::stk_)),
           "hep" => self.set_key(option(OPTION::hep_)),
           "ext" => self.set_key(option(OPTION::ext_)),
           "i8" => self.set_key(form(FORM::i8_)),
           "i16" => self.set_key(form(FORM::i16_)),
           "i32" => self.set_key(form(FORM::i32_)),
           "i64" => self.set_key(form(FORM::i64_)),
           "ia" => self.set_key(form(FORM::ia_)),
           "u8" => self.set_key(form(FORM::u8_)),
           "u16" => self.set_key(form(FORM::u16_)),
           "u32" => self.set_key(form(FORM::u32_)),
           "u64" => self.set_key(form(FORM::u64_)),
           "ua" => self.set_key(form(FORM::ua_)),
           "f32" => self.set_key(form(FORM::f32_)),
           "f64" => self.set_key(form(FORM::f64_)),
           "fa" => self.set_key(form(FORM::fa_)),
           _ => self.set_key(ident(None)),
       }
   }

   fn push(&mut self, code: &mut text2::Text) {
       self.con.push_str(&code.curr_char().to_string());
   }

   fn bump(&mut self, code: &mut text2::Text) {
       code.bump();
       self.con.push_str(&code.curr_char().to_string());
   }

//    fn combine(&mut self, other: &Element) {
//        self.con.push_str(&other.con);
//        self.loc.longer(&other.loc.len())
//    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}  {}", self.loc, self.key, self.con)
    }
}

pub struct Elements {
    elem: Box<dyn Iterator<Item = Element>>,
    win: (Vec<Element>, Element, Vec<Element>),
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
    // pub fn iter(&self) -> Box<dyn Iterator<Item = Element>> {
    //     Box::new(self.elem)
    // }
    pub fn curr(&self) -> Element {
        self.win.1.clone()
    }
    pub fn next_vec(&self) -> Vec<Element> {
        self.win.2.clone()
    }
    pub fn next(&self) -> Element { 
        self.next_vec()[0].clone() 
    }
    pub fn prev_vec(&self) -> Vec<Element> {
        let mut rev = self.win.0.clone();
        rev.reverse();
        rev
    }
    pub fn prev(&self) -> Element { 
        self.prev_vec()[0].clone() 
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
    let mut txt = Box::new(text2::Text::init(dir));
    std::iter::from_fn(move || {
        if let Some(v) = txt.bump() {
            let loc = v.1.1.clone();
            let mut result = Element::empty(illegal, loc.clone(), String::new());
            result.loc.new_word();
            if v.1.0 == '/' && (v.2[0].0 == '/' || v.2[0].0 == '*') {
                result.comment(&mut txt);
            } else if is_eol(&v.1.0) {
                result.endline(&mut txt, false);
            } else if is_space(&v.1.0) {
                result.space(&mut txt);
            } else if v.1.0 == '"' || v.1.0 == '\'' || v.1.0 == '`' {
                result.encap(&mut txt);
            } else if is_digit(&v.1.0) {
                result.digit(&mut txt);
            } else if is_symbol(&v.1.0) {
                result.symbol(&mut txt);
            } else if is_alpha(&v.1.0) {
                result.alpha(&mut txt);
            }
            // let (row, col) = (loc.row(), loc.col());
            // result.loc.adjust(row, col);
            return Some(result);
        }
        None
    })
}
