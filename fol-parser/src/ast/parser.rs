// AST Parser Implementation for FOL

use super::{AstNode, Literal};
use fol_lexer::token::LITERAL;
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
        let mut declarations = Vec::new();
        let mut errors: Vec<Box<dyn Glitch>> = Vec::new();

        for _ in 0..100_000 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(error) => {
                    errors.push(error);
                    break;
                }
            };

            let key = token.key();

            if key.is_eof() {
                break;
            }

            if key.is_illegal() {
                errors.push(Box::new(ParseError::from_token(
                    &token,
                    format!("Parser encountered illegal token '{}'", token.con()),
                )));
                if tokens.bump().is_none() {
                    break;
                }
                continue;
            }

            if key.is_ident() {
                declarations.push(AstNode::Identifier {
                    name: token.con().trim().to_string(),
                });
                if tokens.bump().is_none() {
                    break;
                }
                continue;
            }

            if key.is_literal() {
                match self.parse_lexer_literal(&token) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        if errors.is_empty() {
            Ok(AstNode::Program { declarations })
        } else {
            Err(errors)
        }
    }

    fn parse_lexer_literal(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let raw = token.con().trim();

        match token.key() {
            fol_lexer::token::KEYWORD::Literal(LITERAL::Stringy) => {
                Ok(AstNode::Literal(Literal::String(raw.to_string())))
            }
            fol_lexer::token::KEYWORD::Literal(LITERAL::Bool) => match raw {
                "true" => Ok(AstNode::Literal(Literal::Boolean(true))),
                "false" => Ok(AstNode::Literal(Literal::Boolean(false))),
                _ => Ok(AstNode::Identifier {
                    name: raw.to_string(),
                }),
            },
            _ => self.parse_literal(raw),
        }
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
