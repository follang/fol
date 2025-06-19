// STEP 1: Parse functions with integer bodies

use crate::ast::*;
use crate::syntax::lexer;
use crate::types::*;

// Import our parsers
pub mod integer;
pub mod function;

pub struct AstParser {
    errors: Vec<Box<dyn Glitch>>,
}

impl AstParser {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }
    
    /// Parse tokens into an AST - FUNCTIONS WITH INTEGER BODIES
    pub fn parse(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Vec<Box<dyn Glitch>>> {
        // EXPERIMENTAL: Advance past the initial default token
        tokens.bump();
        println!("DEBUG: After initial bump, first token: {:?}", tokens.curr(true));
        let mut declarations = Vec::new();
        
        // Skip whitespace and comments to get to actual content
        while let Ok(current) = tokens.curr(true) {
            if function::is_function(tokens) {
                match function::parse_function(tokens) {
                    Ok(func_node) => {
                        declarations.push(func_node);
                    }
                    Err(err) => {
                        self.errors.push(err);
                        break;
                    }
                }
            } else {
                // If we can't parse a function, that's an error
                let loc = current.loc().clone();
                let src = current.loc().source().clone();
                self.errors.push(catch!(Typo::ParserUnexpected {
                    loc: Some(loc),
                    key1: current.key(),
                    key2: crate::syntax::token::KEYWORD::Keyword(crate::syntax::token::buildin::BUILDIN::Pro),
                    src,
                }));
                break;
            }
            
            // Check if we're at the end
            if let Ok(current) = tokens.curr(true) {
                if matches!(current.key(), crate::syntax::token::KEYWORD::Void(crate::syntax::token::void::VOID::EndFile)) {
                    break;
                }
            } else {
                break; // No more tokens
            }
        }
        
        if self.errors.is_empty() {
            Ok(AstNode::Program { declarations })
        } else {
            Err(self.errors.clone())
        }
    }
    
    pub fn errors(&self) -> &Vec<Box<dyn Glitch>> {
        &self.errors
    }
    
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}
