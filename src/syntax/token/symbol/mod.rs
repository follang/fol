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
        let t;
        match self {
            SYMBOL::CurlyC => { t = Some("}".to_string()); },
            SYMBOL::CurlyO => { t = Some("{".to_string()); },
            SYMBOL::SquarC => { t = Some("]".to_string()); },
            SYMBOL::SquarO => { t = Some("[".to_string()); },
            SYMBOL::RoundC => { t = Some(")".to_string()); },
            SYMBOL::RoundO => { t = Some("(".to_string()); },
            SYMBOL::AngleC => { t = Some(">".to_string()); },
            SYMBOL::AngleO => { t = Some("<".to_string()); },
            SYMBOL::Dot => { t = Some(".".to_string()); },
            SYMBOL::Comma => { t = Some(",".to_string()); },
            SYMBOL::Colon => { t = Some(":".to_string()); },
            SYMBOL::Semi => { t = Some(";".to_string()); },
            SYMBOL::Escape => { t = Some("\\".to_string()); },
            SYMBOL::Pipe => { t = Some("|".to_string()); },
            SYMBOL::Equal => { t = Some("=".to_string()); },
            SYMBOL::Plus => { t = Some("+".to_string()); },
            SYMBOL::Minus => { t = Some("-".to_string()); },
            SYMBOL::Under => { t = Some("_".to_string()); },
            SYMBOL::Star => { t = Some("*".to_string()); },
            SYMBOL::Home => { t = Some("~".to_string()); },
            SYMBOL::Root => { t = Some("/".to_string()); },
            SYMBOL::Percent => { t = Some("%".to_string()); },
            SYMBOL::Carret => { t = Some("^".to_string()); },
            SYMBOL::Query => { t = Some("?".to_string()); },
            SYMBOL::Bang => { t = Some("!".to_string()); },
            SYMBOL::And => { t = Some("&".to_string()); },
            SYMBOL::At => { t = Some("@".to_string()); },
            SYMBOL::Hash => { t = Some("#".to_string()); },
            SYMBOL::Dollar => { t = Some("$".to_string()); },
            SYMBOL::Degree => { t = Some("°".to_string()); },
            SYMBOL::Sign => { t = Some("§".to_string()); },
            SYMBOL::Tik => { t = Some("`".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}{}",
            " SYMBOL   ".black().on_red(),
            match t { 
                Some(val) => { (":".to_string().white().on_black().to_string() + &format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
