// Function parsing - FOL syntax: fun[] name(params): type = { body } and pro name: type = { body }

use crate::ast::*;
use fol_lexer;
use crate::token::*;
use crate::token::literal::LITERAL;
use crate::token::buildin::BUILDIN;
use crate::token::symbol::SYMBOL;
use crate::token::void::VOID;
use fol_types::*;

/// Parse a function or procedure declaration
pub fn parse_function(tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
    if let Ok(current) = tokens.curr(true) {
        match current.key() {
            // Handle: fun[] name(params): type = { body }
            KEYWORD::Keyword(BUILDIN::Fun) => {
                parse_fun_declaration(tokens)
            }
            // Handle: pro name: type = { body }
            KEYWORD::Keyword(BUILDIN::Pro) => {
                parse_pro_declaration(tokens)
            }
            _ => {
                let loc = current.loc().clone();
                let src = current.loc().source().clone();
                Err(catch!(Typo::ParserUnexpected {
                    loc: Some(loc),
                    key1: current.key(),
                    key2: KEYWORD::Keyword(BUILDIN::Pro), // Expecting either fun or pro
                    src,
                }))
            }
        }
    } else {
        Err(catch!(Typo::ParserMissmatch {
            msg: Some("No token to parse".to_string()),
            loc: None,
            src: None,
        }))
    }
}

/// Parse fun[] name(params): type = { body }
fn parse_fun_declaration(tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
    // At this point, we're positioned at 'fun' - consume it first
    tokens.bump(); // consume 'fun'
    
    // Expect: [optional_brackets]
    if let Ok(current) = tokens.curr(true) {
        if let KEYWORD::Symbol(SYMBOL::SquarO) = current.key() {
            tokens.bump(); // consume '['
            // Skip options for now, look for closing bracket
            if let Ok(close_bracket) = tokens.curr(true) {
                if let KEYWORD::Symbol(SYMBOL::SquarC) = close_bracket.key() {
                    tokens.bump(); // consume ']'
                }
            }
        }
    }
    
    // Get function name
    if let Ok(name_token) = tokens.curr(true) {
        if let KEYWORD::Identifier = name_token.key() {
            let name = name_token.con().clone();
            tokens.bump(); // consume name
            
            parse_function_common(tokens, name, true) // true = is function (not procedure)
        } else {
            let loc = name_token.loc().clone();
            let src = name_token.loc().source().clone();
            Err(catch!(Typo::ParserUnexpected {
                loc: Some(loc),
                key1: name_token.key(),
                key2: KEYWORD::Identifier,
                src,
            }))
        }
    } else {
        Err(catch!(Typo::ParserMissmatch {
            msg: Some("Expected function name".to_string()),
            loc: None,
            src: None,
        }))
    }
}

/// Parse pro name: type = { body }
fn parse_pro_declaration(tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
    // At this point, we're positioned at 'pro' - consume it and get the function name
    println!("DEBUG: Before bump, current token: {:?}", tokens.curr(true));
    println!("DEBUG: Next tokens available: {:?}", tokens.next_vec());
    let bumped = tokens.bump(); // consume 'pro'
    println!("DEBUG: Bump returned: {:?}", bumped);
    println!("DEBUG: After bump, current token: {:?}", tokens.curr(true));
    
    // Get procedure name
    if let Ok(name_token) = tokens.curr(true) {
        if let KEYWORD::Identifier = name_token.key() {
            let name = name_token.con().clone();
            println!("DEBUG: About to consume name '{}', current token: {:?}", name, tokens.curr(true));
            tokens.bump(); // consume name
            println!("DEBUG: After consuming name, current token: {:?}", tokens.curr(true));
            
            parse_function_common(tokens, name, false) // false = is procedure
        } else {
            let loc = name_token.loc().clone();
            let src = name_token.loc().source().clone();
            Err(catch!(Typo::ParserUnexpected {
                loc: Some(loc),
                key1: name_token.key(),
                key2: KEYWORD::Identifier,
                src,
            }))
        }
    } else {
        Err(catch!(Typo::ParserMissmatch {
            msg: Some("Expected procedure name".to_string()),
            loc: None,
            src: None,
        }))
    }
}

/// Common parsing for both functions and procedures after getting the name
fn parse_function_common(tokens: &mut lexer::Elements, name: String, is_function: bool) -> Result<AstNode, Box<dyn Glitch>> {
    println!("DEBUG: parse_function_common - name: {}, is_function: {}", name, is_function);
    println!("DEBUG: Current token at start of common: {:?}", tokens.curr(true));
    
    // For functions: expect (params), for procedures: expect : type
    let mut params = vec![];
    
    if is_function {
        // Expect opening parenthesis for function parameters
        if let Ok(paren_token) = tokens.curr(true) {
            if let KEYWORD::Symbol(SYMBOL::RoundO) = paren_token.key() {
                tokens.bump(); // consume '('
                
                // For now, skip parameters, just look for closing paren
                if let Ok(close_paren) = tokens.curr(true) {
                    if let KEYWORD::Symbol(SYMBOL::RoundC) = close_paren.key() {
                        tokens.bump(); // consume ')'
                    }
                }
            }
        }
    }
    
    // Expect : return_type
    println!("DEBUG: Looking for colon, current token: {:?}", tokens.curr(true));
    if let Ok(colon_token) = tokens.curr(true) {
        if let KEYWORD::Symbol(SYMBOL::Colon) = colon_token.key() {
            println!("DEBUG: Found colon, consuming it");
            tokens.bump(); // consume ':'
            
            // Skip return type parsing for now, expect identifier (like 'int')
            println!("DEBUG: Looking for type after colon: {:?}", tokens.curr(true));
            if let Ok(type_token) = tokens.curr(true) {
                if let KEYWORD::Identifier = type_token.key() {
                    println!("DEBUG: Found type, consuming it");
                    tokens.bump(); // consume type
                }
            }
        }
    }
    
    // Expect = { body }
    println!("DEBUG: Looking for equals, current token: {:?}", tokens.curr(true));
    if let Ok(equal_token) = tokens.curr(true) {
        if let KEYWORD::Symbol(SYMBOL::Equal) = equal_token.key() {
            tokens.bump(); // consume '='
            
            // Expect opening brace
            if let Ok(brace_token) = tokens.curr(true) {
                if let KEYWORD::Symbol(SYMBOL::CurlyO) = brace_token.key() {
                    tokens.bump(); // consume '{'
                    
                    // Parse function body
                    let body = parse_function_body(tokens)?;
                    
                    // Expect closing brace
                    if let Ok(close_brace) = tokens.curr(true) {
                        if let KEYWORD::Symbol(SYMBOL::CurlyC) = close_brace.key() {
                            tokens.bump(); // consume '}'
                        }
                    }
                    
                    if is_function {
                        return Ok(AstNode::FunDecl {
                            name,
                            params,
                            return_type: None, // Skip type parsing for now
                            body,
                            options: vec![],
                            generics: vec![],
                        });
                    } else {
                        return Ok(AstNode::ProDecl {
                            name,
                            params,
                            return_type: None, // Skip type parsing for now
                            body,
                            options: vec![],
                            generics: vec![],
                        });
                    }
                }
            }
        }
    }
    
    let current = tokens.curr(true)?;
    let loc = current.loc().clone();
    let src = current.loc().source().clone();
    Err(catch!(Typo::ParserUnexpected {
        loc: Some(loc),
        key1: current.key(),
        key2: KEYWORD::Symbol(SYMBOL::Equal),
        src,
    }))
}

/// Parse function body - this is where integers can live
fn parse_function_body(tokens: &mut lexer::Elements) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
    let mut statements = Vec::new();
    
    // For now, try to parse one integer literal as the body
    if let Ok(current) = tokens.curr(true) {
        if let KEYWORD::Literal(LITERAL::Deciaml) = current.key() {
            let content = current.con().clone();
            let loc = current.loc().clone();
            let src = current.loc().source().clone();
            tokens.bump(); // consume the token
            
            if let Ok(value) = content.parse::<i64>() {
                statements.push(AstNode::Literal(Literal::Integer(value)));
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some(format!("Invalid integer: {}", content)),
                    loc: Some(loc),
                    src,
                }));
            }
        }
    }
    
    Ok(statements)
}

/// Check if current token starts a function or procedure
pub fn is_function(tokens: &lexer::Elements) -> bool {
    if let Ok(current) = tokens.curr(true) { // Changed from false to true
        matches!(current.key(), 
            KEYWORD::Keyword(BUILDIN::Fun) | 
            KEYWORD::Keyword(BUILDIN::Pro)
        )
    } else {
        false
    }
}
