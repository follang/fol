#![allow(dead_code)]


use std::fmt;
use colored::Colorize;
use super::{Glitch, border_up, border_down};
use crate::syntax::point;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flaw {
    GettingWrongPath {
        msg: Option<String>,
    },
    GettingNoEntry {
        msg: Option<String>,
    },
    ReadingEmptyFile {
        msg: Option<String>,
    },
    ReadingBadContent {
        msg: Option<String>,
    },
    InitError {
        msg: Option<String>,
    },
    EndError {
        msg: Option<String>,
    },
}


impl std::error::Error for Flaw  {  }
impl Glitch for Flaw {  }


impl fmt::Display for Flaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (v, s, m);
        let width = 0;
        match self {
            Flaw::GettingWrongPath { msg } => { 
                s = "getting".to_string();
                v = " PATH_DOES_NOT_EXIST ".to_string(); 
                m = msg.as_ref();
            },
            Flaw::GettingNoEntry { msg } => { 
                s = "getting".to_string();
                v = " NO_FILE_FOUND ".to_string(); 
                m = msg.as_ref();
            },
            Flaw::ReadingBadContent { msg } => { 
                s = "reading".to_string();
                v = " NOT_VALID_TEXT_FILE ".to_string(); 
                m = msg.as_ref();
            },
            Flaw::ReadingEmptyFile { msg } => { 
                s = "reading".to_string();
                v = " FILE_IS_EMPTY ".to_string(); 
                m = msg.as_ref();
            },
            Flaw::InitError { msg } => { 
                s = "reading".to_string();
                v = " INIT_ERROR ".to_string(); 
                m = msg.as_ref();
            },
            Flaw::EndError { msg } => { 
                s = "reading".to_string();
                v = " END_ERROR ".to_string(); 
                m = msg.as_ref();
            },
        };
        write!(f, "{} >> {}:{}{}{}",
            " FLAW ".black().on_red(),
            (" ".to_string() + &s + " file ").black().on_white().to_string(), v.on_red().bold().to_string(),
            match m { Some(val) => "\n".to_string() + &val.to_string(), None => "".to_string() },
            border_down("-", " fol --explain err#".to_string() + "001" + " ")
        )
    }
}
