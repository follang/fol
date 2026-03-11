use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LITERAL {
    ANY,
    Stringy,
    Quoted,
    Bool,
    Float,
    Deciaml,
    Hexal,
    Octal,
    Binary,
}

impl fmt::Display for LITERAL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            LITERAL::Stringy => Some("string"),
            LITERAL::Quoted => Some("quoted"),
            LITERAL::Float => Some("float"),
            LITERAL::Bool => Some("bool"),
            LITERAL::Deciaml => Some("decimal"),
            LITERAL::Hexal => Some("hexal"),
            LITERAL::Octal => Some("octal"),
            LITERAL::Binary => Some("binary"),
            _ => None,
        };
        write!(
            f,
            "{}{}:",
            " LITERAL  ".black().on_red(),
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
