#![allow(dead_code)]
#![allow(unused_macros)]

use std::fmt;
// use crate::scan::scanner;
// use crate::scan::reader;
// use crate::scan::locate;
use crate::scan::token;
use crate::scan::stream;
use crate::error::err;

use crate::scan::scanner::SCAN;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BAG {
    vec: Vec<SCAN>,
    prev: SCAN,
    curr: SCAN,
    brac: Vec<token::SYMBOL>,
}

impl BAG {
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

pub fn init(path: &str, e: &mut err::ERROR) -> BAG {
    let mut stream = stream::STREAM::init(path);
    let prev = stream.prev().to_owned();
    let curr = stream.curr().to_owned();
    let mut vec: Vec<SCAN> = Vec::new();
    while !stream.list().is_empty() {
        vec.push(stream.analyze(e).to_owned());
        stream.bump();
    }
    BAG { vec, prev, curr, brac: Vec::new() }
}


#[macro_export]
macro_rules! expect(($e:expr, $p:expr) => (
    match $e {
        $p => { true },
        _ => { false }
    }
));

impl BAG {
    pub fn not_empty(&self) -> bool {
        !self.list().is_empty()
    }
    pub fn bump(&mut self) {
        if self.not_empty(){
            self.prev = self.curr.to_owned();
            self.vec = self.vec[1..].to_vec();
            self.curr = self.vec.get(0).unwrap_or(&stream::zero()).to_owned();
        }
    }
    pub fn jump(&mut self, t: u8) {
        for i in 0..t {
            self.bump()
        }
    }
    pub fn eat(&mut self) {
        if self.curr().key().is_void(){
            self.bump()
        }
    }

    pub fn toend(&mut self) {
        let deep = self.curr().loc().deep();
        loop {
            if (self.is_terminal() && self.curr().loc().deep() <= deep) || (self.curr().key().is_eof()) { break }
            self.bump()
        }
        self.bump();
        self.eat();
    }

    pub fn report(&mut self, s: String, e: &mut err::ERROR) {
        e.report(err::TYPE::parser, &s, self.curr().loc().clone());
        self.toend();
    }

    pub fn next(&self) -> SCAN {
        self.vec.get(1).unwrap_or(&stream::zero()).to_owned()
    }
    pub fn peek(&self, num: usize) -> SCAN {
        self.vec.get(num).unwrap_or(&stream::zero()).to_owned()
    }

    pub fn is_terminal(&self) -> bool {
        self.curr().key().is_terminal()
    }
}

use crate::scan::token::*;
use crate::scan::token::KEYWORD::*;
impl stream::STREAM {
    pub fn analyze(&mut self, e: &mut err::ERROR) -> SCAN {
        let mut result = self.curr().clone();
        if (self.prev().key().is_void() || self.prev().key().is_bracket()) &&
            self.curr().key().is_symbol() && (self.next().key().is_symbol() || self.next().key().is_void()) {
            if self.after_symbol().is_void() || self.after_symbol().is_bracket() {
                while self.next().key().is_symbol(){
                    result.combine(&self.next());
                    self.bump()
                }
            } else { return result }
            match result.con().as_str() {
                "..." => { result.set_key(operator(OPERATOR::ddd_)) }
                ".." => { result.set_key(operator(OPERATOR::dd_)) }
                "=" => { result.set_key(operator(OPERATOR::assign_)) }
                ":=" => { result.set_key(operator(OPERATOR::assign2_)) }
                "=>" => { result.set_key(operator(OPERATOR::flow_)) }
                "->" => { result.set_key(operator(OPERATOR::flow2_)) }
                "+" => { result.set_key(operator(OPERATOR::add_)) }
                "-" => { result.set_key(operator(OPERATOR::subtract_)) }
                "*" => { result.set_key(operator(OPERATOR::multiply_)) }
                "/" => { result.set_key(operator(OPERATOR::divide_)) }
                "<" => { result.set_key(operator(OPERATOR::less_)) }
                ">" => { result.set_key(operator(OPERATOR::greater_)) }
                "==" => { result.set_key(operator(OPERATOR::equal_)) }
                ">=" => { result.set_key(operator(OPERATOR::greatereq_)) }
                "<=" => { result.set_key(operator(OPERATOR::lesseq_)) }
                "+=" => { result.set_key(operator(OPERATOR::addeq_)) }
                "-=" => { result.set_key(operator(OPERATOR::subtracteq_)) }
                "*=" => { result.set_key(operator(OPERATOR::multiplyeq_)) }
                "/=" => { result.set_key(operator(OPERATOR::divideeq_)) }
                "<<" => { result.set_key(operator(OPERATOR::shiftleft_)) }
                ">>" => { result.set_key(operator(OPERATOR::shiftright_)) }
                _ => {}
            }
        } else if self.curr().key().is_ident() {
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
                "exp" => { result.set_key(option(OPTION::exp_)) },
                "nor" => { result.set_key(option(OPTION::nor_)) },
                "hid" => { result.set_key(option(OPTION::hid_)) },
                "stk" => { result.set_key(option(OPTION::stk_)) },
                "hep" => { result.set_key(option(OPTION::hep_)) },
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
        } else if self.curr().key().is_symbol() && self.next().key().is_assign() {
            match result.con().as_str() {
                "~" => { result.set_key(option(OPTION::mut_)) },
                "!" => { result.set_key(option(OPTION::sta_)) },
                "+" => { result.set_key(option(OPTION::exp_)) },
                "-" => { result.set_key(option(OPTION::hid_)) },
                "@" => { result.set_key(option(OPTION::hep_)) },
                _ => { result.set_key(ident) },
            }
        }
        if self.curr().key().is_eol() {
            if self.prev().key().is_nonterm() || self.prev().key().is_operator() || self.next().key().is_dot() {
                result.set_key(void(VOID::endline_(false)))
            }
        }
        result
    }

}

impl fmt::Display for BAG {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.curr())
    }
}
