// FOL Lexer - Multi-stage tokenization pipeline

use fol_stream::CharacterProvider;

pub mod lexer;
pub mod point;
pub mod token;

// Re-export main types
pub use lexer::*;
pub use point::*;
pub use token::*;

/// Trait for token streams with lookahead/lookbehind
pub trait TokenStream {
    type Token;
    type Error;

    fn next(&mut self) -> Option<Result<Self::Token, Self::Error>>;
    fn peek(&self, offset: usize) -> Option<Result<Self::Token, Self::Error>>;
    fn current(&self) -> Option<Result<Self::Token, Self::Error>>;
}

/// Main lexer interface
pub trait Lexer<S: CharacterProvider> {
    type TokenStream: TokenStream;

    fn new(input: S) -> Self;
    fn tokenize(self) -> Self::TokenStream;
}
