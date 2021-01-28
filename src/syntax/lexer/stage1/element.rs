use std::fmt;
use crate::syntax::point;
use crate::syntax::lexer::text;

use crate::types::{Con, Vod};
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
use crate::syntax::token::{help::*, KEYWORD, KEYWORD::*};


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
    pub fn set_loc(&mut self, l: point::Location) { self.loc = l; }
    pub fn con(&self) -> &String { &self.con }
    pub fn set_con(&mut self, c: String) { self.con = c; }

    //checking
    pub fn comment(&mut self, code: &mut text::Text) -> Vod {
        self.con.push_str(&code.curr().0.to_string());
        self.bump(code);
        if code.curr().0 == '/' {
            self.bump(code);
            while !is_eol(&code.peek(0).0) {
                if is_eof(&code.peek(0).0) { break };
                self.bump(code);
            }
        }
        if code.curr().0 == '*' {
            while !(code.curr().0 == '*' && code.peek(0).0 == '/') {
                if is_eof(&code.peek(0).0) { break };
                self.bump(code);
            }
            self.bump(code);
            //TODO: double check
            if is_space(&code.peek(0).0) {
                self.bump(code);
            }
        }
        self.key = comment;
        Ok(())
    }

    pub fn endfile(&mut self, code: &mut text::Text) -> Vod {
        self.key = void(VOID::endfile_);
        // while is_eol(&code.peek(0).0) || is_space(&code.peek(0).0) {
        //     self.loc.new_line();
        //     self.bump(code);
        // }
        self.con = '\0'.to_string();
        Ok(())
    }

    pub fn endline(&mut self, code: &mut text::Text, terminated: bool) -> Vod {
        self.push(code);
        self.key = void(VOID::endline_);
        while is_eol(&code.peek(0).0) || is_space(&code.peek(0).0) {
            self.loc.new_line();
            self.bump(code);
        }
        self.con = " ".to_string();
        Ok(())
    }

    pub fn space(&mut self, code: &mut text::Text) -> Vod {
        self.push(code);
        while is_space(&code.peek(0).0) {
            self.bump(code);
        }
        if is_eol(&code.peek(0).0) {
            self.bump(code);
            self.endline(code, false)?;
            return Ok(());
        }
        self.key = void(VOID::space_);
        self.con = " ".to_string();
        Ok(())
    }

    pub fn digit(&mut self, code: &mut text::Text) -> Vod {
        if code.curr().0 == '0'
            && (code.peek(0).0 == 'x' || code.peek(0).0 == 'o' || code.peek(0).0 == 'b')
        {
            self.push(code);
            if code.peek(0).0 == 'x' {
                self.bump(code);
                self.key = literal(LITERAL::hexal_);
                while is_hex_digit(&code.peek(0).0) {
                    self.bump(code);
                }
            } else if code.peek(0).0 == 'o' {
                self.bump(code);
                self.key = literal(LITERAL::octal_);
                while is_oct_digit(&code.peek(0).0) {
                    self.bump(code);
                }
            } else if code.peek(0).0 == 'b' {
                self.bump(code);
                self.key = literal(LITERAL::binary_);
                while code.peek(0).0 == '0' || code.peek(0).0 == '1' || code.peek(0).0 == '_'
                {
                    self.bump(code);
                }
            }
        } else {
            self.push(code);
            self.key = literal(LITERAL::decimal_);
            while is_digit(&code.peek(0).0) || code.peek(0).0 == '_' {
                self.bump(code);
            }
        }
        Ok(())
    }

    pub fn encap(&mut self, code: &mut text::Text) -> Vod {
        let litsym = code.curr().0;
        if litsym == '`' {
            self.key = orbit;
        } else if litsym == '\'' {
            self.key = literal(LITERAL::char_);
        } else {
            self.key = literal(LITERAL::string_);
        }
        self.push(code);
        while code.peek(0).0 != litsym || (code.peek(0).0 == litsym && code.curr().0 == '\\')
        {
            if code.peek(0).0 != litsym && code.peek(0).0 == '\0' {
                self.key = illegal;
                break;
            }
            self.bump(code);
        }
        self.bump(code);
        Ok(())
    }

    pub fn symbol(&mut self, code: &mut text::Text) -> Vod {
        self.push(code);
        self.key = symbol(SYMBOL::curlyC_);
        match code.curr().0 {
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
        Ok(())
    }

    pub fn alpha(&mut self, code: &mut text::Text) -> Vod {
        let mut con = code.curr().0.to_string();
        self.push(code);
        while is_alpha(&code.peek(0).0) || is_digit(&code.peek(0).0) {
            self.bump(code);
            con.push(code.curr().0.clone());
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
            _ => self.set_key(ident),
        }
        Ok(())
    }

    pub fn push(&mut self, code: &mut text::Text) {
        self.con.push_str(&code.curr().0.to_string());
    }

    pub fn bump(&mut self, code: &mut text::Text) {
        code.bump();
        self.loc.set_len(self.loc.len() + 1);
        self.con.push_str(&code.curr().0.to_string());
    }
}
