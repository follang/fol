// FOL Lexer - Multi-stage tokenization pipeline

pub mod error;
pub mod lexer;
pub mod point;
pub mod token;

pub use error::LexerError;

// Local type aliases replacing fol-types Con/Vod
pub type Con<T> = Result<T, LexerError>;
pub type Vod = Result<(), LexerError>;

// Re-export main types
pub use lexer::*;
pub use point::*;
pub use token::*;
