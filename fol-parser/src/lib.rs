// FOL Parser - AST generation and parsing

use fol_types::*;
use fol_lexer::TokenStream;

pub mod nodes;
pub mod parse;
pub mod ast;

// Re-export main types
pub use nodes::*;
pub use parse::*;
pub use ast::*;

/// Main parser trait
pub trait Parser<T: TokenStream> {
    type AST;
    type Error;
    
    fn new() -> Self;
    fn parse(&mut self, tokens: T) -> Result<Self::AST, Vec<Self::Error>>;
}

/// FOL-specific parser implementation
pub struct FolParser {
    // Parser state
}

impl FolParser {
    pub fn new() -> Self {
        Self {}
    }
}