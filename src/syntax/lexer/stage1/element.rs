use std::fmt;
use crate::types::{Vod, error::*};
use crate::syntax::point;
use crate::syntax::lexer::stage0;

use crate::syntax::token::{
    literal::LITERAL,
    void::VOID,
    symbol::SYMBOL,
    operator::OPERATOR,
    buildin::BUILDIN,
};
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
            key: KEYWORD::void(VOID::endfile_),
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

    pub fn append(&mut self, other: &Element) {
        self.con.push_str(&other.con);
        self.loc.longer(&other.loc.len())
    }

    pub fn analyze(&mut self, mut code: &mut stage0::Elements) -> Vod {
        if code.curr()?.0 == '/' && (code.peek(0)?.0 == '/' || code.peek(0)?.0 == '*') {
            self.comment(&mut code)?;
        } else if is_eof(&code.curr()?.0) {
            self.endfile(&mut code)?;
        } else if is_eol(&code.curr()?.0) {
            self.endline(&mut code)?;
        } else if is_space(&code.curr()?.0) {
            self.space(&mut code)?;
        } else if code.curr()?.0 == '"' || code.curr()?.0 == '\'' || code.curr()?.0 == '`' {
            self.encap(&mut code)?;
        } else if is_digit(&code.curr()?.0) {
            self.digit(&mut code)?;
        } else if is_symbol(&code.curr()?.0) {
            self.symbol(&mut code)?;
        } else if is_alpha(&code.curr()?.0) {
            self.alpha(&mut code)?;
        } else {
            let msg = format!("{} {}", code.curr()?.0, "is not a recognized character");
            return Err(catch!(Flaw::ReadingBadContent{ msg: Some(msg) }));
        }
        Ok(())
    }

    //checking
    pub fn comment(&mut self, code: &mut stage0::Elements) -> Vod {
        self.con.push_str(&code.curr()?.0.to_string());
        self.bump(code)?;
        if code.curr()?.0 == '/' {
            self.bump(code)?;
            while !is_eol(&code.peek(0)?.0) {
                if is_eof(&code.peek(0)?.0) { break };
                self.bump(code)?;
            }
        }
        if code.curr()?.0 == '*' {
            while !(code.curr()?.0 == '*' && code.peek(0)?.0 == '/') {
                if is_eof(&code.peek(0)?.0) { break };
                self.bump(code)?;
            }
            self.bump(code)?;
            //TODO: double check
            if is_space(&code.peek(0)?.0) {
                self.bump(code)?;
            }
        }
        self.key = comment;
        Ok(())
    }

    pub fn endfile(&mut self, _code: &mut stage0::Elements) -> Vod {
        self.key = void(VOID::endfile_);
        // while is_eol(&code.peek(0).0) || is_space(&code.peek(0).0) {
        //     self.loc.new_line();
        //     self.bump(code);
        // }
        self.con = '\0'.to_string();
        Ok(())
    }

    pub fn endline(&mut self, code: &mut stage0::Elements) -> Vod {
        self.push(code)?;
        self.key = void(VOID::endline_);
        while is_eol(&code.peek(0)?.0) || is_space(&code.peek(0)?.0) {
            self.loc.new_line();
            self.bump(code)?;
        }
        self.con = " ".to_string();
        Ok(())
    }

    pub fn space(&mut self, code: &mut stage0::Elements) -> Vod {
        self.push(code)?;
        while is_space(&code.peek(0)?.0) {
            self.bump(code)?;
        }
        if is_eol(&code.peek(0)?.0) {
            self.bump(code)?;
            self.endline(code)?;
            return Ok(());
        }
        self.key = void(VOID::space_);
        self.con = " ".to_string();
        Ok(())
    }

    pub fn digit(&mut self, code: &mut stage0::Elements) -> Vod {
        if code.curr()?.0 == '0'
            && (code.peek(0)?.0 == 'x' || code.peek(0)?.0 == 'o' || code.peek(0)?.0 == 'b')
        {
            self.push(code)?;
            if code.peek(0)?.0 == 'x' {
                self.bump(code)?;
                self.key = literal(LITERAL::hexal_);
                while is_hex_digit(&code.peek(0)?.0) {
                    self.bump(code)?;
                }
            } else if code.peek(0)?.0 == 'o' {
                self.bump(code)?;
                self.key = literal(LITERAL::octal_);
                while is_oct_digit(&code.peek(0)?.0) {
                    self.bump(code)?;
                }
            } else if code.peek(0)?.0 == 'b' {
                self.bump(code)?;
                self.key = literal(LITERAL::binary_);
                while code.peek(0)?.0 == '0' || code.peek(0)?.0 == '1' || code.peek(0)?.0 == '_'
                {
                    self.bump(code)?;
                }
            }
        } else {
            self.push(code)?;
            self.key = literal(LITERAL::decimal_);
            while is_digit(&code.peek(0)?.0) || code.peek(0)?.0 == '_' {
                self.bump(code)?;
            }
        }
        Ok(())
    }

    pub fn encap(&mut self, code: &mut stage0::Elements) -> Vod {
        let litsym = code.curr()?.0;
        if litsym == '`' {
            self.key = operator(OPERATOR::ANY);
        // } else if litsym == '\'' {
            // self.key = makro;
            // self.key = literal(LITERAL::char_);
        } else {
            self.key = literal(LITERAL::string_);
        }
        self.push(code)?;
        while code.peek(0)?.0 != litsym || (code.peek(0)?.0 == litsym && code.curr()?.0 == '\\')
        {
            if code.peek(0)?.0 != litsym && code.peek(0)?.0 == '\0' {
                self.key = illegal;
                break;
            }
            self.bump(code)?;
        }
        self.bump(code)?;
        Ok(())
    }

    pub fn symbol(&mut self, code: &mut stage0::Elements) -> Vod {
        self.push(code)?;
        self.key = symbol(SYMBOL::curlyC_);
        match code.curr()?.0 {
            '{' => {
                self.key = symbol(SYMBOL::curlyO_)
            }
            '}' => {
                self.key = symbol(SYMBOL::curlyC_)
            }
            '[' => {
                self.key = symbol(SYMBOL::squarO_)
            }
            ']' => {
                self.key = symbol(SYMBOL::squarC_)
            }
            '(' => {
                self.key = symbol(SYMBOL::roundO_)
            }
            ')' => {
                self.key = symbol(SYMBOL::roundC_)
            }
            '<' => {
                self.key = symbol(SYMBOL::angleO_)
            }
            '>' => {
                self.key = symbol(SYMBOL::angleC_)
            }
            ';' => self.key = symbol(SYMBOL::semi_),
            '\\' => self.key = symbol(SYMBOL::escape_),
            '.' => self.key = symbol(SYMBOL::dot_),
            ',' => self.key = symbol(SYMBOL::comma_),
            ':' => self.key = symbol(SYMBOL::colon_),
            '|' => self.key = symbol(SYMBOL::pipe_),
            '=' => self.key = symbol(SYMBOL::equal_),
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

    pub fn alpha(&mut self, code: &mut stage0::Elements) -> Vod {
        let mut con = code.curr()?.0.to_string();
        self.push(code)?;
        while is_alpha(&code.peek(0)?.0) || is_digit(&code.peek(0)?.0) {
            self.bump(code)?;
            con.push(code.curr()?.0.clone());
        }
        match self.con().as_str() {
            "use" => self.set_key(buildin(BUILDIN::use_)),
            "def" => self.set_key(buildin(BUILDIN::def_)),
            "var" => self.set_key(buildin(BUILDIN::var_)),
            "con" => self.set_key(buildin(BUILDIN::con_)),
            "fun" => self.set_key(buildin(BUILDIN::fun_)),
            "pro" => self.set_key(buildin(BUILDIN::pro_)),
            "log" => self.set_key(buildin(BUILDIN::log_)),
            "typ" => self.set_key(buildin(BUILDIN::typ_)),
            "ali" => self.set_key(buildin(BUILDIN::ali_)),
            "imp" => self.set_key(buildin(BUILDIN::imp_)),
            "lab" => self.set_key(buildin(BUILDIN::lab_)),
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
            _ => self.set_key(ident),
        }
        Ok(())
    }

    pub fn push(&mut self, code: &mut stage0::Elements) -> Vod {
        self.con.push_str(&code.curr()?.0.to_string());
        Ok(())
    }

    pub fn bump(&mut self, code: &mut stage0::Elements) -> Vod {
        code.bump();
        self.loc.set_len(self.loc.len() + 1);
        self.con.push_str(&code.curr()?.0.to_string());
        Ok(())
    }
}
