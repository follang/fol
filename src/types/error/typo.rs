#![allow(dead_code)]


use std::fmt;
use colored::Colorize;
use crate::syntax::point;
use crate::syntax::token::KEYWORD;
use terminal_size::{Width, Height, terminal_size};
use super::Glitch;
use crate::types::*;
use crate::syntax::index::source::Source;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Typo {
    ParserUnexpected {
        loc: Option<point::Location>,
        src: Source,
        key1: KEYWORD,
        key2: KEYWORD
    },
    ParserMissmatch {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    ParserSpaceRem {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    ParserSpaceAdd {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    ParserTypeDisbalance {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    ParserBodyForbidden {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    ParserNoType {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    ParserNeedsBody {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    ParserManyUnexpected {
        loc: Option<point::Location>,
        src: Source,
        key1: KEYWORD,
        keys: Vec<KEYWORD>
    },
    LexerPrimitiveAccess {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    LexerBracketUnmatch {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
    LexerSpaceAdd {
        msg: Option<String>,
        loc: Option<point::Location>,
        src: Source,
    },
}

impl std::error::Error for Typo  {  }
impl Glitch for Typo {  }

impl fmt::Display for Typo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (v, s, l, m, message, source);
        let mut comma_separated = String::new();
        match self {
            Typo::ParserUnexpected { loc, key1, key2, src } => { 
                v = " UNEXPECTED TOKEN ".to_string(); 
                s = "parsing".to_string();
                l = loc.as_ref();
                message = format!("expected: {} but got {}", key2, key1);
                m = Some(&message);
                source = src;
            },
            Typo::ParserNeedsBody { msg, loc, src } => {
                v = " MISSING DECLARATATION ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::ParserBodyForbidden { msg, loc, src } => { 
                v = " DECLARATATION FORBIDDEN ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::ParserMissmatch { msg, loc, src } => { 
                v = " MISSMATCHED ARGUMENTS ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::ParserSpaceAdd { msg, loc, src } => { 
                v = " MISSING BLANK SPACE ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::ParserSpaceRem { msg, loc, src } => { 
                v = " OBSOLETE BLANK SPACE ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::ParserTypeDisbalance { msg, loc, src } => { 
                v = " DISBALANCE OF TYPES ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::ParserNoType { msg, loc, src } => { 
                v = " MISSING TYPE ANNOTATION ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::ParserManyUnexpected { loc, key1, keys, src } => { 
                v = " UNEXPECTED TOKEN ".to_string(); 
                s = "parsing".to_string();
                l = loc.as_ref();
                for num in &keys[0..keys.len() - 1] {
                    comma_separated.push_str(&num.to_string());
                    comma_separated.push_str(", ");
                }
                comma_separated.push_str(&keys[keys.len() - 1].to_string());
                message = format!("expected one of: {},\ninstead recieved {}", comma_separated, key1);
                m = Some(&message);
                source = src;
            },
            Typo::LexerBracketUnmatch { msg, loc, src } => { 
                v = " UNMATCHED BRACKET ".to_string(); 
                s = "lexing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::LexerSpaceAdd { msg, loc, src } => { 
                v = " MISSING BLANK SPACE ".to_string(); 
                s = "lexing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
            Typo::LexerPrimitiveAccess { msg, loc, src } => { 
                v = " PRIMITIVE_ACCESS ".to_string(); 
                s = "lexing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
                source = src;
            },
        };
        let width = if let Some((Width(w), Height(h))) = terminal_size() { w } else { 5 };
        write!(f, "{} >> {}:{}{}{}\n{}",
            " TYPO ".black().on_red(),
            (" ".to_string() + &s + " stage ").black().on_white().to_string(), v.on_red().bold().to_string(),
            match l { Some(val) => "\n".to_string() + &val.visualize(&source), None => "".to_string() },
            match m { Some(val) => "\n".to_string() + &val.to_string(), None => "".to_string() },
            "-".repeat(width as usize).red()
        )
    }
}
