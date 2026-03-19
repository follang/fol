use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OPERATOR {
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
            OPERATOR::Dotdotdot => "...",
            OPERATOR::Dotdot => "..",
            OPERATOR::Path => "::",
            OPERATOR::Assign => ":=",
            OPERATOR::Flow => "=>",
            OPERATOR::Flow2 => "->",
            OPERATOR::Add => "+",
            OPERATOR::Abstract => "-",
            OPERATOR::Multiply => "*",
            OPERATOR::Divide => "/",
            OPERATOR::Equal => "==",
            OPERATOR::Noteq => "!=",
            OPERATOR::Greateq => ">=",
            OPERATOR::Lesseq => "<=",
            OPERATOR::Addeq => "+=",
            OPERATOR::Subeq => "-=",
            OPERATOR::Multeq => "*=",
            OPERATOR::Diveq => "/=",
            OPERATOR::Lesser => "<<",
            OPERATOR::Greater => ">>",
        };
        write!(
            f,
            "{}{}",
            " OPERATOR ".black().on_red(),
            (":".to_string().white().on_black().to_string() + &format!(" {} ", t))
                .black()
                .on_red()
        )
    }
}
