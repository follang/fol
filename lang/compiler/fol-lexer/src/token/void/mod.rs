use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VOID {
    Space,
    EndLine,
    Boundary,
    EndFile,
}

impl fmt::Display for VOID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            VOID::Space => "space",
            VOID::EndFile => "EOF",
            VOID::EndLine => "eol",
            VOID::Boundary => "boundary",
        };
        write!(
            f,
            "{}{}",
            " VOID     ".black().on_red(),
            (":".to_string().white().on_black().to_string() + &format!(" {} ", t))
                .black()
                .on_red()
        )
    }
}
