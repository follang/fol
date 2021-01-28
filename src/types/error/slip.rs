#![allow(dead_code)]


use std::fmt;
use colored::Colorize;
use super::Glitch;
use crate::syntax::point;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slip {
    UnmatchedBracket {
        msg: Option<String>,
    },
    UnmatchedQuote {
        msg: Option<String>,
    },
    UnfinishedComment {
        msg: Option<String>,
    },
}


impl std::error::Error for Slip  {  }
impl Glitch for Slip {  }

impl fmt::Display for Slip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (v, s, m);
        match self {
            Slip::UnmatchedBracket { msg } => { 
                s = "getting".to_string();
                v = " PATH_DOES_NOT_EXIST ".to_string(); 
                m = msg.as_ref();
            },
            Slip::UnmatchedQuote { msg } => { 
                s = "getting".to_string();
                v = " NO_FILE_FOUND ".to_string(); 
                m = msg.as_ref();
            },
            Slip::UnfinishedComment { msg } => { 
                s = "reading".to_string();
                v = " FILE_IS_EMPTY ".to_string(); 
                m = msg.as_ref();
            },
        };
        write!(f, "\n{} >> {}:{}{}",
            " SLIP ".black().on_red(),
            (" ".to_string() + &s + " file ").black().on_white().to_string(), v.on_red().bold().to_string(),
            match m { Some(val) => "\n".to_string() + &val.to_string(), None => "".to_string() },
        )
    }
}
