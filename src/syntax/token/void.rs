use colored::Colorize;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum VOID {
    ANY,
    space_,
    endline_,
    endfile_,
}

impl fmt::Display for VOID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t;
        match self {
            VOID::space_ => { t = Some("space".to_string()); },
            VOID::endfile_ => { t = Some("eol".to_string()); },
            VOID::endline_ => { t = Some("EOF".to_string()); },
            _ => { t = None },
        };
        write!(f, "{}: {}",
            " VOID     ".black().on_red(),
            match t { 
                Some(val) => { (format!(" {} ", val)).black().on_red().to_string() }, 
                None => "".to_string()
            },
        )
    }
}
