#![allow(dead_code)]

use std::fmt;
// use crate::scan::scanner;
// use crate::scan::reader;
// use crate::scan::locate;
use crate::scan::token;
use crate::scan::stream;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LEXEME {
    vec: Vec<SCAN>,
    prev: SCAN,
    curr: SCAN,
}

impl LEXEME {
    pub fn list(&self) -> &Vec<SCAN> {
        &self.vec
    }
    pub fn curr(&self) -> &SCAN {
        &self.curr
    }
    pub fn prev(&self) -> &SCAN {
        &self.prev
    }
}

impl LEXEME {
    pub fn init(path: &str ) -> Self {
        let mut stream = stream::STREAM::init(path);
        let prev = stream.prev().to_owned();
        let curr = stream.curr().to_owned();
        let mut vec: Vec<SCAN> = Vec::new();
        while !stream.list().is_empty() {
            vec.push(stream.analyze().to_owned());
            stream.bump();
        }
        LEXEME { vec, prev, curr }
    }

    pub fn not_empty(&self) -> bool {
        !self.list().is_empty()
    }

    pub fn bump(&mut self) {
        if !self.vec.is_empty(){
            self.prev = self.curr.to_owned();
            self.vec = self.vec[1..].to_vec();
            self.curr = self.vec.get(0).unwrap_or(&stream::zero()).to_owned();
        }
    }
    pub fn next(&self) -> SCAN {
        self.vec.get(1).unwrap_or(&stream::zero()).to_owned()
    }
    pub fn peek(&self, num: usize) -> SCAN {
        self.vec.get(num).unwrap_or(&stream::zero()).to_owned()
    }
}

use crate::scan::token::*;
use crate::scan::token::KEYWORD::*;
impl stream::STREAM {
    pub fn analyze(&mut self) -> SCAN {
        let mut result = self.curr().clone();
        if (self.prev().key().is_void() || self.prev().key().is_bracket()) &&
            self.curr().key().is_symbol() && self.next().key().is_symbol() {
            if self.after_symbol().is_void() || self.after_symbol().is_bracket() {
                while self.next().key().is_symbol(){
                    result.combine(&self.next());
                    self.bump()
                }
            } else { return result }
            match result.con().as_str() {
                "..." => { result.set_key(operator(OPERATOR::ddd_)) }
                ".." => { result.set_key(operator(OPERATOR::dd_)) }
                _ => { result.set_key(illegal) }
            }
        }
        if self.curr().key().is_ident() {
            match result.con().as_str() {
                "use" => { result.set_key(assign(ASSIGN::use_)) },
                "def" => { result.set_key(assign(ASSIGN::def_)) },
                "var" => { result.set_key(assign(ASSIGN::var_)) },
                "fun" => { result.set_key(assign(ASSIGN::fun_)) },
                "pro" => { result.set_key(assign(ASSIGN::pro_)) },
                "log" => { result.set_key(assign(ASSIGN::log_)) },
                "typ" => { result.set_key(assign(ASSIGN::typ_)) },
                "ali" => { result.set_key(assign(ASSIGN::ali_)) },
                "int" => { result.set_key(types(TYPE::int_)) },
                "flt" => { result.set_key(types(TYPE::flt_)) },
                "chr" => { result.set_key(types(TYPE::chr_)) },
                "bol" => { result.set_key(types(TYPE::bol_)) },
                "arr" => { result.set_key(types(TYPE::arr_)) },
                "vec" => { result.set_key(types(TYPE::vec_)) },
                "seq" => { result.set_key(types(TYPE::seq_)) },
                "mat" => { result.set_key(types(TYPE::mat_)) },
                "set" => { result.set_key(types(TYPE::set_)) },
                "map" => { result.set_key(types(TYPE::map_)) },
                "axi" => { result.set_key(types(TYPE::axi_)) },
                "tab" => { result.set_key(types(TYPE::tab_)) },
                "str" => { result.set_key(types(TYPE::str_)) },
                "num" => { result.set_key(types(TYPE::num_)) },
                "ptr" => { result.set_key(types(TYPE::ptr_)) },
                "err" => { result.set_key(types(TYPE::err_)) },
                "opt" => { result.set_key(types(TYPE::opt_)) },
                "nev" => { result.set_key(types(TYPE::nev_)) },
                "uni" => { result.set_key(types(TYPE::uni_)) },
                "any" => { result.set_key(types(TYPE::any_)) },
                "non" => { result.set_key(types(TYPE::non_)) },
                "nil" => { result.set_key(types(TYPE::nil_)) },
                "rec" => { result.set_key(types(TYPE::rec_)) },
                "ent" => { result.set_key(types(TYPE::ent_)) },
                "blu" => { result.set_key(types(TYPE::blu_)) },
                "std" => { result.set_key(types(TYPE::std_)) },
                "loc" => { result.set_key(types(TYPE::loc_)) },
                "url" => { result.set_key(types(TYPE::url_)) },
                "blk" => { result.set_key(types(TYPE::blk_)) },
                "not" => { result.set_key(buildin(BUILDIN::not_)) },
                "or" => { result.set_key(buildin(BUILDIN::or_)) },
                "xor" => { result.set_key(buildin(BUILDIN::xor_)) },
                "and" => { result.set_key(buildin(BUILDIN::and_)) },
                "nand" => { result.set_key(buildin(BUILDIN::nand_)) },
                "as" => { result.set_key(buildin(BUILDIN::as_)) },
                "if" => { result.set_key(buildin(BUILDIN::if_)) },
                "when" => { result.set_key(buildin(BUILDIN::when_)) },
                "loop" => { result.set_key(buildin(BUILDIN::loop_)) },
                "is" => { result.set_key(buildin(BUILDIN::is_)) },
                "has" => { result.set_key(buildin(BUILDIN::has_)) },
                "in" => { result.set_key(buildin(BUILDIN::in_)) },
                "case" => { result.set_key(buildin(BUILDIN::case_)) },
                "this" => { result.set_key(buildin(BUILDIN::this_)) },
                "self" => { result.set_key(buildin(BUILDIN::self_)) },
                "break" => { result.set_key(buildin(BUILDIN::break_)) },
                "return" => { result.set_key(buildin(BUILDIN::return_)) },
                "yeild" => { result.set_key(buildin(BUILDIN::yeild_)) },
                "panic" => { result.set_key(buildin(BUILDIN::panic_)) },
                "report" => { result.set_key(buildin(BUILDIN::report_)) },
                "check" => { result.set_key(buildin(BUILDIN::check_)) },
                "assert" => { result.set_key(buildin(BUILDIN::assert_)) },
                "where" => { result.set_key(buildin(BUILDIN::where_)) },
                "true" => { result.set_key(buildin(BUILDIN::true_)) },
                "false" => { result.set_key(buildin(BUILDIN::false_)) },
                "each" => { result.set_key(buildin(BUILDIN::each_)) },
                "for" => { result.set_key(buildin(BUILDIN::for_)) },
                "do" => { result.set_key(buildin(BUILDIN::do_)) },
                "go" => { result.set_key(buildin(BUILDIN::go_)) },
                "get" => { result.set_key(buildin(BUILDIN::get_)) },
                "let" => { result.set_key(buildin(BUILDIN::let_)) },
                "mut" => { result.set_key(option(OPTION::mut_)) },
                "imu" => { result.set_key(option(OPTION::imu_)) },
                "sta" => { result.set_key(option(OPTION::sta_)) },
                "rea" => { result.set_key(option(OPTION::rea_)) },
                "exp" => { result.set_key(option(OPTION::exp_)) },
                "nor" => { result.set_key(option(OPTION::nor_)) },
                "hid" => { result.set_key(option(OPTION::hid_)) },
                "i8" => { result.set_key(form(FORM::i8_)) },
                "i16" => { result.set_key(form(FORM::i16_)) },
                "i32" => { result.set_key(form(FORM::i32_)) },
                "i64" => { result.set_key(form(FORM::i64_)) },
                "ia" => { result.set_key(form(FORM::ia_)) },
                "u8" => { result.set_key(form(FORM::u8_)) },
                "u16" => { result.set_key(form(FORM::u16_)) },
                "u32" => { result.set_key(form(FORM::u32_)) },
                "u64" => { result.set_key(form(FORM::u64_)) },
                "ua" => { result.set_key(form(FORM::ua_)) },
                "f32" => { result.set_key(form(FORM::f32_)) },
                "f64" => { result.set_key(form(FORM::f64_)) },
                "fa" => { result.set_key(form(FORM::fa_)) },
                _ => { result.set_key(ident) },
            }
        }
        result
    }
    pub fn symbol(&mut self) {

    }
}

impl fmt::Display for LEXEME {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.curr())
    }
}

