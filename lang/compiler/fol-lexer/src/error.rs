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

// Temporary Glitch impl — will be removed in Slice 5
impl fol_types::Glitch for LexerError {
    fn clone_box(&self) -> Box<dyn fol_types::Glitch> {
        Box::new(self.clone())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// Allow `?` to convert LexerError into Box<dyn Glitch> for parser compatibility
impl From<LexerError> for Box<dyn fol_types::Glitch> {
    fn from(e: LexerError) -> Self {
        Box::new(e)
    }
}
