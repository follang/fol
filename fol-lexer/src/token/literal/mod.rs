use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LITERAL {
    ANY,
    Stringy,
    Bool,
    Float,
    Deciaml,
    Hexal,
    Octal,
    Binary,
}

impl fmt::Display for LITERAL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            LITERAL::Stringy => { t = Some("string".to_string()); },
            LITERAL::Float => { t = Some("float".to_string()); },
            LITERAL::Bool => { t = Some("bool".to_string()); },
            LITERAL::Deciaml => { t = Some("decimal".to_string()); },
            LITERAL::Hexal => { t = Some("hexal".to_string()); },
            LITERAL::Octal => { t = Some("octal".to_string()); },
            LITERAL::Binary => { t = Some("binary".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}{}:",
            " LITERAL  ".black().on_red(),
            match t { 
                Some(val) => { (":".to_string().white().on_black().to_string() + &format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
