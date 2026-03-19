use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SYMBOL {
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
}

impl fmt::Display for SYMBOL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            SYMBOL::CurlyC => "}",
            SYMBOL::CurlyO => "{",
            SYMBOL::SquarC => "]",
            SYMBOL::SquarO => "[",
            SYMBOL::RoundC => ")",
            SYMBOL::RoundO => "(",
            SYMBOL::AngleC => ">",
            SYMBOL::AngleO => "<",
            SYMBOL::Dot => ".",
            SYMBOL::Comma => ",",
            SYMBOL::Colon => ":",
            SYMBOL::Semi => ";",
            SYMBOL::Escape => "\\",
            SYMBOL::Pipe => "|",
            SYMBOL::Equal => "=",
            SYMBOL::Plus => "+",
            SYMBOL::Minus => "-",
            SYMBOL::Under => "_",
            SYMBOL::Star => "*",
            SYMBOL::Home => "~",
            SYMBOL::Root => "/",
            SYMBOL::Percent => "%",
            SYMBOL::Carret => "^",
            SYMBOL::Query => "?",
            SYMBOL::Bang => "!",
            SYMBOL::And => "&",
            SYMBOL::At => "@",
            SYMBOL::Hash => "#",
            SYMBOL::Dollar => "$",
            SYMBOL::Degree => "°",
            SYMBOL::Sign => "§",
        };
        write!(
            f,
            "{}{}",
            " SYMBOL   ".black().on_red(),
            (":".to_string().white().on_black().to_string() + &format!(" {} ", t))
                .black()
                .on_red()
        )
    }
}
