use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LITERAL {
    ANY,
    string_,
    bool_,
    float_,
    decimal_,
    hexal_,
    octal_,
    binary_,
}

impl fmt::Display for LITERAL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            LITERAL::string_ => { t = Some("string".to_string()); },
            LITERAL::float_ => { t = Some("float".to_string()); },
            LITERAL::bool_ => { t = Some("bool".to_string()); },
            LITERAL::decimal_ => { t = Some("decimal".to_string()); },
            LITERAL::hexal_ => { t = Some("hexal".to_string()); },
            LITERAL::octal_ => { t = Some("octal".to_string()); },
            LITERAL::binary_ => { t = Some("binary".to_string()); },
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
