#![allow(dead_code)]


use std::fmt;
use colored::Colorize;
use crate::syntax::point;
use crate::syntax::token::KEYWORD;
use super::Glitch;
use crate::types::*;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Typo {
    ParserUnexpected {
        loc: Option<point::Location>,
        key1: KEYWORD,
        key2: KEYWORD
    },
    ParserMissmatch {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    ParserSpaceRem {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    ParserSpaceAdd {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    ParserTypeDisbalance {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    ParserBodyForbidden {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    ParserNoType {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    ParserNeedsBody {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    ParserManyUnexpected {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    LexerPrimitiveAccess {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    LexerBracketUnmatch {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
    LexerSpaceAdd {
        msg: Option<String>,
        loc: Option<point::Location>,
    },
}

impl std::error::Error for Typo  {  }
impl Glitch for Typo {  }

impl fmt::Display for Typo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (v, s, l, m);
        let message;
        match self {
            Typo::ParserUnexpected { loc, key1, key2 } => { 
                v = " UNEXPECTED TOKEN ".to_string(); 
                s = "parsing".to_string();
                l = loc.as_ref();
                message = format!("expected: {} but got {}", key2, key1);
                m = Some(&message);
            },
            Typo::ParserNeedsBody { msg, loc } => {
                v = " MISSING DECLARATATION ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserBodyForbidden { msg, loc } => { 
                v = " DECLARATATION FORBIDDEN ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserMissmatch { msg, loc } => { 
                v = " MISSMATCHED ARGUMENTS ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserSpaceAdd { msg, loc } => { 
                v = " MISSING BLANK SPACE ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserSpaceRem { msg, loc } => { 
                v = " OBSOLETE BLANK SPACE ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserTypeDisbalance { msg, loc } => { 
                v = " DISBALANCE OF TYPES ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserNoType { msg, loc } => { 
                v = " MISSING TYPE ANNOTATION ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserManyUnexpected { msg, loc } => { 
                v = " UNEXPECTED TOKEN ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::LexerBracketUnmatch { msg, loc } => { 
                v = " UNMATCHED BRACKET ".to_string(); 
                s = "lexing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::LexerSpaceAdd { msg, loc } => { 
                v = " MISSING BLANK SPACE ".to_string(); 
                s = "lexing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::LexerPrimitiveAccess { msg, loc } => { 
                v = " PRIMITIVE_ACCESS ".to_string(); 
                s = "lexing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
        };
        write!(f, "\n{} >> {}:{}{}{}",
            " TYPO ".black().on_red(),
            (" ".to_string() + &s + " stage ").black().on_white().to_string(), v.on_red().bold().to_string(),
            match l { Some(val) => "\n".to_string() + &val.visualize(), None => "".to_string() },
            match m { Some(val) => "\n".to_string() + &val.to_string(), None => "".to_string() },
        )
    }
}
