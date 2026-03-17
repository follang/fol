use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SYMBOL {
    ANY,
    RoundO,
    RoundC,
    SquarO,
    SquarC,
    CurlyO,
    CurlyC,
    AngleO,
    AngleC,
    Dot,
    Comma,
    Colon,
    Semi,
    Escape,
    Pipe,
    Equal,
    Greater,
    Less,
    Plus,
    Minus,
    Under,
    Star,
    Home,
    Root,
    Percent,
    Carret,
    Query,
    Bang,
    And,
    At,
    Hash,
    Dollar,
    Degree,
    Sign,
    Tik,
}

impl fmt::Display for SYMBOL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            SYMBOL::CurlyC => Some("}"),
            SYMBOL::CurlyO => Some("{"),
            SYMBOL::SquarC => Some("]"),
            SYMBOL::SquarO => Some("["),
            SYMBOL::RoundC => Some(")"),
            SYMBOL::RoundO => Some("("),
            SYMBOL::AngleC => Some(">"),
            SYMBOL::AngleO => Some("<"),
            SYMBOL::Dot => Some("."),
            SYMBOL::Comma => Some(","),
            SYMBOL::Colon => Some(":"),
            SYMBOL::Semi => Some(";"),
            SYMBOL::Escape => Some("\\"),
            SYMBOL::Pipe => Some("|"),
            SYMBOL::Equal => Some("="),
            SYMBOL::Plus => Some("+"),
            SYMBOL::Minus => Some("-"),
            SYMBOL::Under => Some("_"),
            SYMBOL::Star => Some("*"),
            SYMBOL::Home => Some("~"),
            SYMBOL::Root => Some("/"),
            SYMBOL::Percent => Some("%"),
            SYMBOL::Carret => Some("^"),
            SYMBOL::Query => Some("?"),
            SYMBOL::Bang => Some("!"),
            SYMBOL::And => Some("&"),
            SYMBOL::At => Some("@"),
            SYMBOL::Hash => Some("#"),
            SYMBOL::Dollar => Some("$"),
            SYMBOL::Degree => Some("°"),
            SYMBOL::Sign => Some("§"),
            SYMBOL::Tik => Some("`"),
            _ => None,
        };
        write!(
            f,
            "{}{}",
            " SYMBOL   ".black().on_red(),
            match t {
                Some(val) => {
                    (":".to_string().white().on_black().to_string() + &format!(" {} ", val))
                        .black()
                        .on_red()
                        .to_string()
                }
                None => "".to_string(),
            },
        )
    }
}
