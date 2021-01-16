#![allow(dead_code)]

use std::fmt;
use crate::syntax::point;
use crate::syntax::scan::reader;
use crate::syntax::scan::parts;
use crate::syntax::scan::token;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SCAN {
    key: token::KEYWORD,
    loc: point::Location,
    con: String,
}

impl SCAN {
    pub fn new(key: token::KEYWORD, loc: point::Location, con: String) -> Self {
        SCAN { key, loc, con }
    }
    pub fn zero(name: &str) -> Self {
        let key = token::KEYWORD::void(token::VOID::endfile_);
        let loc = point::Location::new(name.to_string(), name.to_string(), 0, 0, 0, 0);
        SCAN {
            key,
            loc,
            con: String::new(),
        }
    }
    pub fn key(&self) -> &token::KEYWORD {
        &self.key
    }
    pub fn loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn con(&self) -> &String {
        &self.con
    }
    pub fn set_key(&mut self, k: token::KEYWORD) {
        self.key = k;
    }
    pub fn log(&self, msg: &str) {
        println!(
            " {} [{:>2} {:>2}] \t key:{} \t\t{}",
            msg,
            self.loc().row(),
            self.loc().col(),
            self.key(),
            self.con()
        );
    }
}

impl fmt::Display for SCAN {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {: <20} {}", self.loc, self.key, self.con)
    }
}

/// Creates a vector that produces tokens from the input string.
pub fn vectorize(red: &mut reader::READER) -> Vec<SCAN> {
    let mut vec: Vec<SCAN> = Vec::new();
    let mut loc = point::Location::init(&red.path(), &red.name());
    let mut part = parts::PART::init(&red.data);
    while part.not_eof() {
        let token = part.scanning(&mut loc);
        vec.push(token)
    }
    vec
}

use crate::syntax::scan::token::KEYWORD::*;
use crate::syntax::scan::token::*;
impl parts::PART {
    /// Parses a token from the input string.
    fn scanning(&mut self, loc: &mut point::Location) -> SCAN {
        let mut result = SCAN::new(illegal, loc.clone(), String::new());
        result.loc.new_word();
        self.bump(&mut result.loc);
        //ignore comments
        if self.curr_char() == '/' && (self.next_char() == '/' || self.next_char() == '*') {
            result.comment(self);
        } else if is_eol(&self.curr_char()) {
            result.endline(self, false);
        } else if is_space(&self.curr_char()) {
            result.space(self);
        //TODO: fix comment here
        } else if self.curr_char() == '"' || self.curr_char() == '\'' || self.curr_char() == '`' {
            result.encap(self);
        } else if is_digit(&self.curr_char()) {
            result.digit(self);
        } else if is_symbol(&self.curr_char()) {
            result.symbol(self);
        } else if is_alpha(&self.curr_char()) {
            result.alpha(self);
        }
        let (row, col) = (loc.row(), loc.col());
        *loc = result.loc.clone();
        result.loc.adjust(row, col);
        return result;
    }
}

impl SCAN {
    pub fn combine(&mut self, other: &SCAN) {
        self.con.push_str(&other.con);
        self.loc.longer(&other.loc.len())
    }

    fn comment(&mut self, part: &mut parts::PART) {
        let mut con = part.curr_char().to_string();
        self.bump(part);
        if part.curr_char() == '/' {
            con.push(part.curr_char().clone());
            self.bump(part);
            while !is_eol(&part.next_char()) {
                if is_eof(&part.next_char()) { break };
                con.push(part.curr_char().clone());
                self.bump(part);
            }
        }
        if part.curr_char() == '*' {
            con.push(part.curr_char().clone());
            con.push(part.next_char().clone());
            self.bump(part);
            while part.curr_char() != '*' && part.next_char() != '/' {
                if is_eol(&part.next_char()) { self.loc.new_line(); }
                else if is_eof(&part.next_char()) { break };
                con.push(part.next_char().clone());
                self.bump(part);
            }
            con.push(part.next_char().clone());
            self.bump(part);
            //TODO: double check
            if is_space(&part.next_char()) {
                self.bump(part);
            }
        }
        self.key = comment;
        self.con = con;
    }
    fn endline(&mut self, part: &mut parts::PART, terminated: bool) {
        self.push_curr(part);
        self.loc.new_line();
        self.key = void(VOID::endline_);
        while is_eol(&part.next_char()) || is_space(&part.next_char()) {
            if is_eol(&part.next_char()) {
                self.loc.new_line();
            }
            self.bump(part);
        }
        self.con = " ".to_string();
    }
    fn space(&mut self, part: &mut parts::PART) {
        self.push_curr(part);
        while is_space(&part.next_char()) {
            self.bump(part);
        }
        if is_eol(&part.next_char()) {
            self.bump(part);
            self.endline(part, false);
            return;
        }
        self.key = void(VOID::space_);
        self.con = " ".to_string();
    }
    fn digit(&mut self, part: &mut parts::PART) {
        if part.curr_char() == '0'
            && (part.next_char() == 'x' || part.next_char() == 'o' || part.next_char() == 'b')
        {
            self.push_curr(part);
            if part.next_char() == 'x' {
                self.bump(part);
                self.key = literal(LITERAL::hexal_);
                while is_hex_digit(&part.next_char()) {
                    self.bump(part);
                }
            } else if part.next_char() == 'o' {
                self.bump(part);
                self.key = literal(LITERAL::octal_);
                while is_oct_digit(&part.next_char()) {
                    self.bump(part);
                }
            } else if part.next_char() == 'b' {
                self.bump(part);
                self.key = literal(LITERAL::binary_);
                while part.next_char() == '0' || part.next_char() == '1' || part.next_char() == '_'
                {
                    self.bump(part);
                }
            }
        } else {
            self.push_curr(part);
            self.key = literal(LITERAL::decimal_);
            while is_digit(&part.next_char()) || part.next_char() == '_' {
                self.bump(part);
            }
        }
    }
    fn encap(&mut self, part: &mut parts::PART) {
        let litsym = part.curr_char();
        if litsym == '`' {
            self.key = comment;
        } else if litsym == '\'' {
            self.key = literal(LITERAL::char_);
        } else {
            self.key = literal(LITERAL::string_);
        }
        self.push_curr(part);
        while part.next_char() != litsym || (part.next_char() == litsym && part.curr_char() == '\\')
        {
            if part.next_char() != litsym && part.next_char() == '\0' {
                self.key = illegal;
                break;
            } else if is_eol(&part.next_char()) {
                self.loc.new_line();
            }
            self.bump(part);
        }
        self.bump(part);
    }
    fn symbol(&mut self, part: &mut parts::PART) {
        self.push_curr(part);
        self.key = symbol(SYMBOL::curlyC_);
        match part.curr_char() {
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
    fn alpha(&mut self, part: &mut parts::PART) {
        self.push_curr(part);
        while is_alpha(&part.next_char()) || is_digit(&part.next_char()) {
            part.bump(&mut self.loc);
            self.push_curr(part);
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

    fn push_curr(&mut self, part: &mut parts::PART) {
        self.con.push_str(&part.curr_char().to_string());
    }

    fn bump(&mut self, part: &mut parts::PART) {
        part.bump(&mut self.loc);
        self.con.push_str(&part.curr_char().to_string());
    }
}

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

