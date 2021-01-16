#![allow(dead_code)]


use std::fmt;
use colored::Colorize;
use crate::syntax::point;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Typo {
    ParserUnexpected {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserMissmatch {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserSpaceRem {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserSpaceAdd {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserTypeDisbalance {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserBodyForbidden {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserNoType {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserNeedsBody {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    ParserManyUnexpected {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    LexerPrimitiveAccess {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    LexerBracketUnmatch {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
    LexerSpaceAdd {
        msg: Option<&'static str>,
        loc: Option<point::Location>,
    },
}

impl fmt::Display for Typo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (v, s);
        let (mut l, mut m) = (None, None);
        match self {
            Typo::ParserUnexpected { msg, loc } => { 
                v = " UNEXPECTED TOKEN ".to_string(); 
                s = "parsing".to_string();
                m = msg.as_ref();
                l = loc.as_ref();
            },
            Typo::ParserNeedsBody { msg, loc } => {
                v = " MISSING DECLARATATION ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::ParserBodyForbidden { msg, loc } => { 
                v = " DECLARATATION FORBIDDEN ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::ParserMissmatch { msg, loc } => { 
                v = " MISSMATCHED ARGUMENTS ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::ParserSpaceAdd { msg, loc } => { 
                v = " MISSING BLANK SPACE ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::ParserSpaceRem { msg, loc } => { 
                v = " OBSOLETE BLANK SPACE ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::ParserTypeDisbalance { msg, loc } => { 
                v = " DISBALANCE OF TYPES ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::ParserNoType { msg, loc } => { 
                v = " MISSING TYPE ANNOTATION ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::ParserManyUnexpected { msg, loc } => { 
                v = " UNEXPECTED TOKEN ".to_string(); 
                s = "parsing".to_string();
            },
            Typo::LexerBracketUnmatch { msg, loc } => { 
                v = " UNMATCHED BRACKET ".to_string(); 
                s = "lexing".to_string();
            },
            Typo::LexerSpaceAdd { msg, loc } => { 
                v = " MISSING BLANK SPACE ".to_string(); 
                s = "lexing".to_string();
            },
            Typo::LexerPrimitiveAccess { msg, loc } => { 
                v = " PRIMITIVE_ACCESS ".to_string(); 
                s = "lexing".to_string();
            },
        };
        write!(f, "\n\n{} >> {}:{}{}{}",
            " TYPO ".black().on_red(),
            (" ".to_string() + &s + " stage ").black().bold().on_white().to_string(), v.on_red().to_string(),
            match l { Some(val) => "\n".to_string() + &val.visualize(), None => "".to_string() },
            match m { Some(val) => "\n".to_string() + &val.to_string(), None => "".to_string() },
        )
    }
}
impl std::error::Error for Typo  {}
impl Glitch for Typo  {
    // fn report(typ: Self) -> Self {
    //     typ
    // }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Read {
    GettingMissingFile {
        msg: Option<&'static str>,
    },
    GettingNoEntry {
        msg: Option<&'static str>,
    },
    ReadingEmptyFile {
        msg: Option<&'static str>,
    },
    ReadingBadContent {
        msg: Option<&'static str>,
    },
}

pub trait Glitch: std::error::Error + fmt::Display {
    // fn report(typ: Self) -> Self;
}

pub type Cont<T> = Result<T, dyn Glitch>;
pub type Void = Result<(), dyn Glitch>;
