// Integer parsing - JUST integers, nothing else!

use crate::ast::*;
use crate::syntax::lexer;
use crate::syntax::token::*;
use crate::syntax::token::literal::LITERAL;
use crate::types::*;

/// Parse ONLY an integer literal
pub fn parse_integer(tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
    if let Ok(current) = tokens.curr(true) {
        if let KEYWORD::Literal(LITERAL::Deciaml) = current.key() {
            let content = current.con().clone();
            let loc = current.loc().clone();
            let src = current.loc().source().clone();
            tokens.bump(); // consume the token
            
            if let Ok(value) = content.parse::<i64>() {
                Ok(AstNode::Literal(Literal::Integer(value)))
            } else {
                Err(catch!(Typo::ParserMissmatch {
                    msg: Some(format!("Invalid integer: {}", content)),
                    loc: Some(loc),
                    src,
                }))
            }
        } else {
            let loc = current.loc().clone();
            let src = current.loc().source().clone();
            Err(catch!(Typo::ParserUnexpected {
                loc: Some(loc),
                key1: current.key(),
                key2: KEYWORD::Literal(LITERAL::Deciaml),
                src,
            }))
        }
    } else {
        Err(catch!(Typo::ParserMissmatch {
            msg: Some("No token to parse".to_string()),
            loc: None,
            src: None,
        }))
    }
}

/// Check if current token is an integer
pub fn is_integer(tokens: &lexer::Elements) -> bool {
    if let Ok(current) = tokens.curr(false) {
        matches!(current.key(), KEYWORD::Literal(LITERAL::Deciaml))
    } else {
        false
    }
}
