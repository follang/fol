// Basic error types for FOL

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Flaw {
    ReadingBadContent { msg: Option<String> },
    GettingNoEntry { msg: Option<String> },
    GettingWrongPath { msg: Option<String> },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Typo {
    LexerSpaceAdd { msg: Option<String> },
    ParserMissmatch { msg: Option<String> },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Slip {
    // Parser errors
}

impl std::error::Error for Flaw {}
impl std::error::Error for Typo {}
impl std::error::Error for Slip {}

impl crate::Glitch for Flaw {
    fn clone_box(&self) -> Box<dyn crate::Glitch> {
        Box::new(self.clone())
    }
}
impl crate::Glitch for Typo {
    fn clone_box(&self) -> Box<dyn crate::Glitch> {
        Box::new(self.clone())
    }
}
impl crate::Glitch for Slip {
    fn clone_box(&self) -> Box<dyn crate::Glitch> {
        Box::new(self.clone())
    }
}

impl fmt::Display for Flaw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Flaw::ReadingBadContent { msg } => {
                write!(f, "ReadingBadContent: {}", msg.as_ref().unwrap_or(&"".to_string()))
            }
            Flaw::GettingNoEntry { msg } => {
                write!(f, "GettingNoEntry: {}", msg.as_ref().unwrap_or(&"".to_string()))
            }
            Flaw::GettingWrongPath { msg } => {
                write!(f, "GettingWrongPath: {}", msg.as_ref().unwrap_or(&"".to_string()))
            }
        }
    }
}

impl fmt::Display for Typo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Typo::LexerSpaceAdd { msg } => {
                write!(f, "LexerSpaceAdd: {}", msg.as_ref().unwrap_or(&"".to_string()))
            }
            Typo::ParserMissmatch { msg } => {
                write!(f, "ParserMissmatch: {}", msg.as_ref().unwrap_or(&"".to_string()))
            }
        }
    }
}

impl fmt::Display for Slip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Slip")
    }
}