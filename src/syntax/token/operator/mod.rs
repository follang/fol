use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OPERATOR {
    ANY,
    Dotdot,
    Dotdotdot,
    Path,
    Assign,
    Flow,
    Flow2,
    Add,
    Abstract,
    Multiply,
    Divide,
    Equal,
    Noteq,
    Greateq,
    Lesseq,
    Addeq,
    Subeq,
    Multeq,
    Diveq,
    Lesser,
    Greater,
}

impl fmt::Display for OPERATOR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            OPERATOR::Dotdotdot => { t = Some("...".to_string()); },
            OPERATOR::Dotdot => { t = Some("..".to_string()); },
            OPERATOR::Path => { t = Some("::".to_string()); },
            OPERATOR::Assign => { t = Some(":=".to_string()); },
            OPERATOR::Flow => { t = Some("=>".to_string()); },
            OPERATOR::Flow2 => { t = Some("->".to_string()); },
            OPERATOR::Add => { t = Some("+".to_string()); },
            OPERATOR::Abstract => { t = Some("-".to_string()); },
            OPERATOR::Multiply => { t = Some("*".to_string()); },
            OPERATOR::Divide => { t = Some("/".to_string()); },
            OPERATOR::Equal => { t = Some("==".to_string()); },
            OPERATOR::Noteq => { t = Some("!=".to_string()); },
            OPERATOR::Greateq => { t = Some(">=".to_string()); },
            OPERATOR::Lesseq => { t = Some("<=".to_string()); },
            OPERATOR::Addeq => { t = Some("+=".to_string()); },
            OPERATOR::Subeq => { t = Some("-=".to_string()); },
            OPERATOR::Multeq => { t = Some("*=".to_string()); },
            OPERATOR::Diveq => { t = Some("/=".to_string()); },
            OPERATOR::Lesser => { t = Some("<<".to_string()); },
            OPERATOR::Greater => { t = Some(">>".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}{}",
            " OPERATOR ".black().on_red(),
            match t { 
                Some(val) => { (":".to_string().white().on_black().to_string() + &format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
