use crate::lexer::stage0;
use crate::point;
use fol_types::{catch, error::*, Vod};
use std::fmt;

use crate::token::{
    buildin::BUILDIN, literal::LITERAL, operator::OPERATOR, symbol::SYMBOL, void::VOID,
};
use crate::token::{help::*, KEYWORD, KEYWORD::*};

// Stage 1 owns first-pass token classification only.
// It converts raw characters into the initial token families before later
// folding and numeric disambiguation refine the stream.
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

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}  {}", self.loc, self.key, self.con)
    }
}

impl Element {
    pub fn init(key: KEYWORD, loc: point::Location, con: String) -> Self {
        Self { key, loc, con }
    }
    pub fn key(&self) -> &KEYWORD {
        &self.key
    }
    pub fn set_key(&mut self, k: KEYWORD) {
        self.key = k;
    }
    pub fn loc(&self) -> &point::Location {
        &self.loc
    }
    pub fn set_loc(&mut self, l: point::Location) {
        self.loc = l;
    }
    pub fn con(&self) -> &String {
        &self.con
    }
    pub fn set_con(&mut self, c: String) {
        self.con = c;
    }

    pub fn append(&mut self, other: &Element) {
        self.con.push_str(&other.con);
        self.loc.longer(&other.loc.len())
    }

    pub fn analyze(&mut self, code: &mut stage0::Elements) -> Vod {
        if code.curr()?.0 == '/' && (code.peek(0)?.0 == '/' || code.peek(0)?.0 == '*') {
            self.comment(code)?;
        } else if is_eof(&code.curr()?.0) {
            self.endfile(code)?;
        } else if is_eol(&code.curr()?.0) {
            self.endline(code)?;
        } else if is_space(&code.curr()?.0) {
            self.space(code)?;
        } else if code.curr()?.0 == '"' || code.curr()?.0 == '\'' || code.curr()?.0 == '`' {
            self.encap(code)?;
        } else if is_digit(&code.curr()?.0) {
            self.digit(code)?;
        } else if is_symbol(&code.curr()?.0) {
            self.symbol(code)?;
        } else if is_alpha(&code.curr()?.0) {
            self.alpha(code)?;
        } else {
            let msg = format!("{} {}", code.curr()?.0, "is not a recognized character");
            return Err(catch!(Flaw::ReadingBadContent { msg: Some(msg) }));
        }
        Ok(())
    }

    //checking
    pub fn comment(&mut self, code: &mut stage0::Elements) -> Vod {
        // Ordinary comments and doc-comment spellings share the same lexer path for now.
        // Front-end hardening keeps them fully ignorable and leaves any future doc-comment
        // semantics to a later phase instead of surfacing a separate token family today.
        self.con.push_str(&code.curr()?.0.to_string());
        self.bump(code)?;
        if code.curr()?.0 == '/' {
            self.bump(code)?;
            while !is_eol(&code.peek(0)?.0) {
                if is_eof(&code.peek(0)?.0) {
                    break;
                };
                self.bump(code)?;
            }
        }
        if code.curr()?.0 == '*' {
            while !(code.curr()?.0 == '*' && code.peek(0)?.0 == '/') {
                if is_eof(&code.peek(0)?.0) {
                    break;
                };
                self.bump(code)?;
            }
            self.bump(code)?;
            //TODO: double check
            if is_space(&code.peek(0)?.0) {
                self.bump(code)?;
            }
        }
        self.key = Comment;
        self.con = " ".to_string();
        self.space(code)?;
        Ok(())
    }

    pub fn endfile(&mut self, _code: &mut stage0::Elements) -> Vod {
        self.key = Void(VOID::EndFile);
        // while is_eol(&code.peek(0)?.0) || is_space(&code.peek(0)?.0) {
        //     self.loc.new_line();
        //     self.bump(code);
        // }
        self.con = '\0'.to_string();
        Ok(())
    }

    pub fn endline(&mut self, code: &mut stage0::Elements) -> Vod {
        self.push(code)?;
        self.key = Void(VOID::EndLine);
        while is_eol(&code.peek(0)?.0) || is_space(&code.peek(0)?.0) {
            // self.loc.new_line();
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
        self.key = Void(VOID::Space);
        self.con = " ".to_string();
        Ok(())
    }

    pub fn digit(&mut self, code: &mut stage0::Elements) -> Vod {
        // Numeric scanning currently stops at the supported literal body.
        // Identifier suffixes such as an imaginary-unit marker stay separate so the
        // front end does not silently invent a numeric family it does not support.
        if code.curr()?.0 == '0'
            && matches!(code.peek(0)?.0, 'x' | 'X' | 'o' | 'O' | 'b' | 'B')
        {
            self.push(code)?;
            if matches!(code.peek(0)?.0, 'x' | 'X') {
                self.bump(code)?;
                self.key = Literal(LITERAL::Hexal);
                while is_hex_digit(&code.peek(0)?.0) {
                    self.bump(code)?;
                }
            } else if matches!(code.peek(0)?.0, 'o' | 'O') {
                self.bump(code)?;
                self.key = Literal(LITERAL::Octal);
                while is_oct_digit(&code.peek(0)?.0) {
                    self.bump(code)?;
                }
            } else if matches!(code.peek(0)?.0, 'b' | 'B') {
                self.bump(code)?;
                self.key = Literal(LITERAL::Binary);
                while code.peek(0)?.0 == '0' || code.peek(0)?.0 == '1' || code.peek(0)?.0 == '_' {
                    self.bump(code)?;
                }
            }
        } else {
            self.push(code)?;
            self.key = Literal(LITERAL::Deciaml);
            while is_digit(&code.peek(0)?.0) || code.peek(0)?.0 == '_' {
                self.bump(code)?;
            }
        }
        Ok(())
    }

    pub fn encap(&mut self, code: &mut stage0::Elements) -> Vod {
        let litsym = code.curr()?.0;
        if litsym == '`' {
            self.key = Operator(OPERATOR::ANY);
        } else if litsym == '\'' {
            self.key = Literal(LITERAL::Quoted);
        } else {
            self.key = Literal(LITERAL::Stringy);
        }
        self.push(code)?;
        // Escapes stay verbatim in the token payload during front-end hardening.
        // The lexer only keeps the quoted span intact and does not validate
        // escape spellings beyond finding the matching closing delimiter.
        // If EOF arrives first, every quoted form on this path degrades to the same
        // parser-visible Illegal token instead of producing delimiter-specific behavior.
        while code.peek(0)?.0 != litsym || (code.peek(0)?.0 == litsym && code.curr()?.0 == '\\') {
            if code.peek(0)?.0 != litsym && code.peek(0)?.0 == '\0' {
                self.key = Illegal;
                break;
            }
            self.bump(code)?;
        }
        self.bump(code)?;
        Ok(())
    }

    pub fn symbol(&mut self, code: &mut stage0::Elements) -> Vod {
        self.push(code)?;
        self.key = Symbol(SYMBOL::CurlyC);
        match code.curr()?.0 {
            '{' => self.key = Symbol(SYMBOL::CurlyO),
            '}' => self.key = Symbol(SYMBOL::CurlyC),
            '[' => self.key = Symbol(SYMBOL::SquarO),
            ']' => self.key = Symbol(SYMBOL::SquarC),
            '(' => self.key = Symbol(SYMBOL::RoundO),
            ')' => self.key = Symbol(SYMBOL::RoundC),
            '<' => self.key = Symbol(SYMBOL::AngleO),
            '>' => self.key = Symbol(SYMBOL::AngleC),
            ';' => self.key = Symbol(SYMBOL::Semi),
            '\\' => self.key = Symbol(SYMBOL::Escape),
            '.' => self.key = Symbol(SYMBOL::Dot),
            ',' => self.key = Symbol(SYMBOL::Comma),
            ':' => self.key = Symbol(SYMBOL::Colon),
            '|' => self.key = Symbol(SYMBOL::Pipe),
            '=' => self.key = Symbol(SYMBOL::Equal),
            '+' => self.key = Symbol(SYMBOL::Plus),
            '-' => self.key = Symbol(SYMBOL::Minus),
            '_' => self.key = Symbol(SYMBOL::Under),
            '*' => self.key = Symbol(SYMBOL::Star),
            '~' => self.key = Symbol(SYMBOL::Home),
            '/' => self.key = Symbol(SYMBOL::Root),
            '%' => self.key = Symbol(SYMBOL::Percent),
            '^' => self.key = Symbol(SYMBOL::Carret),
            '?' => self.key = Symbol(SYMBOL::Query),
            '!' => self.key = Symbol(SYMBOL::Bang),
            '&' => self.key = Symbol(SYMBOL::And),
            '@' => self.key = Symbol(SYMBOL::At),
            '#' => self.key = Symbol(SYMBOL::Hash),
            '$' => self.key = Symbol(SYMBOL::Dollar),
            '°' => self.key = Symbol(SYMBOL::Degree),
            '§' => self.key = Symbol(SYMBOL::Sign),
            _ => self.key = Illegal,
        }
        Ok(())
    }

    pub fn alpha(&mut self, code: &mut stage0::Elements) -> Vod {
        let mut con = code.curr()?.0.to_string();
        self.push(code)?;
        while is_alpha(&code.peek(0)?.0) || is_digit(&code.peek(0)?.0) {
            self.bump(code)?;
            con.push(code.curr()?.0);
        }
        match self.con().as_str() {
            "use" => self.set_key(Keyword(BUILDIN::Use)),
            "def" => self.set_key(Keyword(BUILDIN::Def)),
            "seg" => self.set_key(Keyword(BUILDIN::Seg)),
            "var" => self.set_key(Keyword(BUILDIN::Var)),
            "log" => self.set_key(Keyword(BUILDIN::Log)),
            "con" => self.set_key(Keyword(BUILDIN::Con)),
            "fun" => self.set_key(Keyword(BUILDIN::Fun)),
            "pro" => self.set_key(Keyword(BUILDIN::Pro)),
            "typ" => self.set_key(Keyword(BUILDIN::Typ)),
            "std" => self.set_key(Keyword(BUILDIN::Std)),
            "ali" => self.set_key(Keyword(BUILDIN::Ali)),
            "imp" => self.set_key(Keyword(BUILDIN::Imp)),
            "lab" => self.set_key(Keyword(BUILDIN::Lab)),
            "not" => self.set_key(Keyword(BUILDIN::Not)),
            "or" => self.set_key(Keyword(BUILDIN::Or)),
            "xor" => self.set_key(Keyword(BUILDIN::Xor)),
            "and" => self.set_key(Keyword(BUILDIN::And)),
            "nand" => self.set_key(Keyword(BUILDIN::Nand)),
            "as" => self.set_key(Keyword(BUILDIN::As)),
            "cast" => self.set_key(Keyword(BUILDIN::Cast)),
            "if" => self.set_key(Keyword(BUILDIN::If)),
            "else" => self.set_key(Keyword(BUILDIN::Else)),
            "while" => self.set_key(Keyword(BUILDIN::While)),
            "when" => self.set_key(Keyword(BUILDIN::When)),
            "loop" => self.set_key(Keyword(BUILDIN::Loop)),
            "is" => self.set_key(Keyword(BUILDIN::Is)),
            "has" => self.set_key(Keyword(BUILDIN::Has)),
            "in" => self.set_key(Keyword(BUILDIN::In)),
            "on" => self.set_key(Keyword(BUILDIN::On)),
            "of" => self.set_key(Keyword(BUILDIN::Of)),
            "case" => self.set_key(Keyword(BUILDIN::Case)),
            "this" => self.set_key(Keyword(BUILDIN::This)),
            "self" => self.set_key(Keyword(BUILDIN::Selfi)),
            "break" => self.set_key(Keyword(BUILDIN::Break)),
            "return" => self.set_key(Keyword(BUILDIN::Return)),
            "yeild" => self.set_key(Keyword(BUILDIN::Yeild)),
            "panic" => self.set_key(Keyword(BUILDIN::Panic)),
            "report" => self.set_key(Keyword(BUILDIN::Report)),
            "check" => self.set_key(Keyword(BUILDIN::Check)),
            "assert" => self.set_key(Keyword(BUILDIN::Assert)),
            "where" => self.set_key(Keyword(BUILDIN::Where)),
            "true" => self.set_key(Keyword(BUILDIN::True)),
            "false" => self.set_key(Keyword(BUILDIN::False)),
            "each" => self.set_key(Keyword(BUILDIN::Each)),
            "for" => self.set_key(Keyword(BUILDIN::For)),
            "do" => self.set_key(Keyword(BUILDIN::Do)),
            "go" => self.set_key(Keyword(BUILDIN::Go)),
            "let" => self.set_key(Keyword(BUILDIN::Let)),
            "async" => self.set_key(Keyword(BUILDIN::Async)),
            "await" => self.set_key(Keyword(BUILDIN::Await)),
            "select" => self.set_key(Keyword(BUILDIN::Select)),
            _ => self.set_key(Identifier),
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
