use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LexerError {
    ReadingBadContent(String),
    GettingNoEntry(String),
    GettingWrongPath(String),
    LexerSpaceAdd(String),
    ParserMismatch(String),
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexerError::ReadingBadContent(msg) => write!(f, "ReadingBadContent: {msg}"),
            LexerError::GettingNoEntry(msg) => write!(f, "GettingNoEntry: {msg}"),
            LexerError::GettingWrongPath(msg) => write!(f, "GettingWrongPath: {msg}"),
            LexerError::LexerSpaceAdd(msg) => write!(f, "LexerSpaceAdd: {msg}"),
            LexerError::ParserMismatch(msg) => write!(f, "ParserMismatch: {msg}"),
        }
    }
}

impl std::error::Error for LexerError {}
