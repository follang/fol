// AST Parser Implementation for FOL

use super::{AstNode, Literal};
use fol_types::*;
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
    file: Option<String>,
    line: usize,
    column: usize,
    length: usize,
}

impl ParseError {
    pub fn from_token(token: &fol_lexer::lexer::stage3::element::Element, message: String) -> Self {
        let loc = token.loc();
        Self {
            message,
            file: loc.source().map(|src| src.path(true)),
            line: loc.row(),
            column: loc.col(),
            length: loc.len(),
        }
    }

    pub fn file(&self) -> Option<String> {
        self.file.clone()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl Glitch for ParseError {
    fn clone_box(&self) -> Box<dyn Glitch> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Simple AST Parser for FOL
pub struct AstParser {
    // Parser state can be added here later
}

impl Default for AstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser {
    pub fn new() -> Self {
        Self {}
    }

    /// Parse a token stream into an AST
    pub fn parse(
        &mut self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Vec<Box<dyn Glitch>>> {
        if let Ok(token) = tokens.curr(false) {
            if token.key().is_illegal() {
                let error = ParseError::from_token(
                    &token,
                    format!("Parser encountered illegal token '{}'", token.con()),
                );
                return Err(vec![Box::new(error)]);
            }
        }

        // For now, return a simple program node
        // This is a minimal implementation to get compilation working
        Ok(AstNode::Program {
            declarations: vec![],
        })
    }

    /// Parse a simple literal for testing
    pub fn parse_literal(&self, value: &str) -> Result<AstNode, Box<dyn Glitch>> {
        // Simple integer parsing for testing
        if let Ok(int_val) = value.parse::<i64>() {
            return Ok(AstNode::Literal(Literal::Integer(int_val)));
        }

        // Simple string parsing
        if value.starts_with('"') && value.ends_with('"') {
            let string_val = value[1..value.len() - 1].to_string();
            return Ok(AstNode::Literal(Literal::String(string_val)));
        }

        // Default to identifier
        Ok(AstNode::Identifier {
            name: value.to_string(),
        })
    }
}
