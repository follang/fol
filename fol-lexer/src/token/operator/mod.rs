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
        let t = match self {
            OPERATOR::Dotdotdot => Some("..."),
            OPERATOR::Dotdot => Some(".."),
            OPERATOR::Path => Some("::"),
            OPERATOR::Assign => Some(":="),
            OPERATOR::Flow => Some("=>"),
            OPERATOR::Flow2 => Some("->"),
            OPERATOR::Add => Some("+"),
            OPERATOR::Abstract => Some("-"),
            OPERATOR::Multiply => Some("*"),
            OPERATOR::Divide => Some("/"),
            OPERATOR::Equal => Some("=="),
            OPERATOR::Noteq => Some("!="),
            OPERATOR::Greateq => Some(">="),
            OPERATOR::Lesseq => Some("<="),
            OPERATOR::Addeq => Some("+="),
            OPERATOR::Subeq => Some("-="),
            OPERATOR::Multeq => Some("*="),
            OPERATOR::Diveq => Some("/="),
            OPERATOR::Lesser => Some("<<"),
            OPERATOR::Greater => Some(">>"),
            _ => None,
        };
        write!(
            f,
            "{}{}",
            " OPERATOR ".black().on_red(),
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
