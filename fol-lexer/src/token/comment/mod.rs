use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum COMMENT {
    Backtick,
    Doc,
    SlashLine,
    SlashBlock,
}

impl fmt::Display for COMMENT {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            COMMENT::Backtick => "backtick",
            COMMENT::Doc => "doc",
            COMMENT::SlashLine => "slash-line",
            COMMENT::SlashBlock => "slash-block",
        };

        write!(
            f,
            "{}{}:",
            " COMMENT  ".black().on_red(),
            (":".to_string().white().on_black().to_string() + &format!(" {} ", label))
                .black()
                .on_red(),
        )
    }
}
