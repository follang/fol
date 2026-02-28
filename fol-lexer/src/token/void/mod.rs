use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VOID {
    ANY,
    Space,
    EndLine,
    EndFile,
}

impl fmt::Display for VOID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match self {
            VOID::Space => Some("space"),
            VOID::EndFile => Some("EOF"),
            VOID::EndLine => Some("eol"),
            _ => None,
        };
        write!(
            f,
            "{}{}",
            " VOID     ".black().on_red(),
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
