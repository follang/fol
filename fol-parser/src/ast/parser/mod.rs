// STEP 1: Parse functions with integer bodies

use crate::ast::*;
use fol_lexer;
use fol_types::*;

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
    
    /// Parse tokens into an AST - TOP LEVEL DECLARATIONS
    pub fn parse(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Vec<Box<dyn Glitch>>> {
        // EXPERIMENTAL: Advance past the initial default token
        tokens.bump();
        println!("DEBUG: After initial bump, first token: {:?}", tokens.curr(true));
        let mut declarations = Vec::new();
        
        // Parse top-level declarations
        loop {
            // Skip whitespace and comments first
            if let Err(e) = self.skip_whitespace_and_comments(tokens) {
                return Err(vec![e]);
            }
            
            // Check if we have a meaningful token
            let current = match tokens.curr(true) {
                Ok(token) => token,
                Err(_) => break, // No more tokens
            };
            
            println!("DEBUG: Parsing top-level, current token: {:?}", current.key());
            
            // Check for end of file
            if matches!(current.key(), crate::token::KEYWORD::Void(crate::token::void::VOID::EndFile)) {
                break;
            }
            
            // Try to parse different top-level declarations
            let parsed = if self.is_use_declaration(tokens) {
                self.parse_use_declaration(tokens)
            } else if self.is_type_declaration(tokens) {
                self.parse_type_declaration(tokens)
            } else if self.is_const_declaration(tokens) {
                self.parse_const_declaration(tokens)
            } else if function::is_function(tokens) {
                function::parse_function(tokens)
            } else if self.is_alias_declaration(tokens) {
                self.parse_alias_declaration(tokens)
            } else if self.is_impl_declaration(tokens) {
                self.parse_impl_declaration(tokens)
            } else if self.is_segment_declaration(tokens) {
                self.parse_segment_declaration(tokens)
            } else {
                // Unknown declaration - skip with error
                let loc = current.loc().clone();
                let src = current.loc().source().clone();
                self.errors.push(catch!(Typo::ParserUnexpected {
                    loc: Some(loc),
                    key1: current.key(),
                    key2: crate::token::KEYWORD::Keyword(crate::token::buildin::BUILDIN::Use),
                    src,
                }));
                break;
            };
            
            match parsed {
                Ok(decl_node) => {
                    declarations.push(decl_node);
                }
                Err(err) => {
                    self.errors.push(err);
                    break;
                }
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
    
    // ===== HELPER FUNCTIONS =====
    
    /// Skip whitespace, comments, and other non-meaningful tokens
    fn skip_whitespace_and_comments(&self, tokens: &mut lexer::Elements) -> Result<(), Box<dyn Glitch>> {
        while let Ok(current) = tokens.curr(true) {
            match current.key() {
                crate::token::KEYWORD::Comment |
                crate::token::KEYWORD::Void(crate::token::void::VOID::Space) |
                crate::token::KEYWORD::Void(crate::token::void::VOID::EndLine) => {
                    tokens.bump(); // Skip these tokens
                }
                _ => break, // Found a meaningful token
            }
        }
        Ok(())
    }
    
    // ===== TOP-LEVEL DECLARATION PARSERS =====
    
    /// Check if current token starts a use declaration: use[]
    fn is_use_declaration(&self, tokens: &lexer::Elements) -> bool {
        if let Ok(current) = tokens.curr(true) {
            matches!(current.key(), crate::token::KEYWORD::Keyword(crate::token::buildin::BUILDIN::Use))
        } else {
            false
        }
    }
    
    /// Parse use declaration: use[options] name: type = { path } or use[options] ( name: type = { path }; ... )
    fn parse_use_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        tokens.bump(); // consume 'use'
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Skip [] options for now
        if let Ok(current) = tokens.curr(true) {
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::SquarO) = current.key() {
                tokens.bump(); // consume '['
                // Skip to closing bracket
                while let Ok(current) = tokens.curr(true) {
                    if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::SquarC) = current.key() {
                        tokens.bump(); // consume ']'
                        break;
                    }
                    tokens.bump();
                }
            }
        }
        
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Check if this is a grouped declaration with parentheses
        if let Ok(current) = tokens.curr(true) {
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::RoundO) = current.key() {
                // This is a grouped use declaration: use[] ( name: type = { path }; ... )
                return self.parse_grouped_use_declaration(tokens);
            }
        }
        
        // Parse single use declaration: use[options] name: type = { path }
        self.parse_single_use_declaration(tokens)
    }
    
    /// Parse single use declaration: name: type = { path }
    fn parse_single_use_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        // Get name
        let name = if let Ok(name_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Identifier = name_token.key() {
                let name = name_token.con().clone();
                tokens.bump();
                name
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some("Expected identifier in use declaration".to_string()),
                    loc: Some(name_token.loc().clone()),
                    src: name_token.loc().source().clone(),
                }));
            }
        } else {
            return Err(catch!(Typo::ParserMissmatch {
                msg: Some("Expected identifier in use declaration".to_string()),
                loc: None,
                src: None,
            }));
        };
        
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Expect colon
        if let Ok(colon_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::Colon) = colon_token.key() {
                tokens.bump(); // consume ':'
                if let Err(e) = self.skip_whitespace_and_comments(tokens) {
                return Err(vec![e]);
            }
                
                // Get type (loc, url, std, mod, etc.)
                let path_type = if let Ok(type_token) = tokens.curr(true) {
                    if let crate::token::KEYWORD::Identifier = type_token.key() {
                        let type_name = type_token.con().clone();
                        tokens.bump();
                        match type_name.as_str() {
                            "loc" => FolType::Named { name: "loc".to_string() },
                            "url" => FolType::Named { name: "url".to_string() },
                            "std" => FolType::Named { name: "std".to_string() },
                            "mod" => FolType::Module { name: "mod".to_string() },
                            _ => FolType::Named { name: type_name },
                        }
                    } else {
                        return Err(catch!(Typo::ParserMissmatch {
                            msg: Some("Expected type after colon in use declaration".to_string()),
                            loc: Some(type_token.loc().clone()),
                            src: type_token.loc().source().clone(),
                        }));
                    }
                } else {
                    return Err(catch!(Typo::ParserMissmatch {
                        msg: Some("Expected type after colon in use declaration".to_string()),
                        loc: None,
                        src: None,
                    }));
                };
                
                // Skip = { path } for now - just consume until we hit a recognized top-level token
                if let Err(e) = self.skip_to_next_declaration(tokens) {
                    return Err(vec![e]);
                }
                
                Ok(AstNode::UseDecl {
                    options: vec![],
                    name,
                    path_type,
                    path: "unknown".to_string(),
                })
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some("Expected ':' after name in use declaration".to_string()),
                    loc: Some(colon_token.loc().clone()),
                    src: colon_token.loc().source().clone(),
                }));
            }
        } else {
            return Err(catch!(Typo::ParserMissmatch {
                msg: Some("Expected ':' after name in use declaration".to_string()),
                loc: None,
                src: None,
            }));
        }
    }
    
    /// Parse grouped use declaration: ( name: type = { path }; name: type = { path }; ... )
    fn parse_grouped_use_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        tokens.bump(); // consume '('
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // For now, just skip the entire group and treat as a single use declaration
        // In a full implementation, you'd parse each declaration inside the parentheses
        let mut paren_depth = 1;
        while let Ok(current) = tokens.curr(true) {
            match current.key() {
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::RoundO) => {
                    paren_depth += 1;
                    tokens.bump();
                }
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::RoundC) => {
                    paren_depth -= 1;
                    tokens.bump();
                    if paren_depth == 0 {
                        break; // Found closing paren
                    }
                }
                _ => {
                    tokens.bump();
                }
            }
        }
        
        // Return a placeholder use declaration for the group
        Ok(AstNode::UseDecl {
            options: vec![],
            name: "grouped_use".to_string(),
            path_type: FolType::Named { name: "group".to_string() },
            path: "grouped".to_string(),
        })
    }
    
    /// Check if current token starts a type declaration: typ or ~typ
    fn is_type_declaration(&self, tokens: &lexer::Elements) -> bool {
        if let Ok(current) = tokens.curr(true) {
            matches!(current.key(), 
                crate::token::KEYWORD::Keyword(crate::token::buildin::BUILDIN::Typ) |
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::Home)
            )
        } else {
            false
        }
    }
    
    /// Parse typ declaration: [~]typ[options] name: type = { definition }
    fn parse_type_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        // Handle optional ~ prefix
        if let Ok(current) = tokens.curr(true) {
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::Home) = current.key() {
                tokens.bump(); // consume '~'
                if let Err(e) = self.skip_whitespace_and_comments(tokens) {
                return Err(vec![e]);
            }
            }
        }
        
        tokens.bump(); // consume 'typ'
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Skip [] options
        if let Ok(current) = tokens.curr(true) {
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::SquarO) = current.key() {
                tokens.bump(); // consume '['
                // Skip to closing bracket
                while let Ok(current) = tokens.curr(true) {
                    if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::SquarC) = current.key() {
                        tokens.bump(); // consume ']'
                        break;
                    }
                    tokens.bump();
                }
            }
        }
        
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Get name  
        let name = if let Ok(name_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Identifier = name_token.key() {
                let name = name_token.con().clone();
                println!("DEBUG: Found name: {}", name);
                tokens.bump();
                if let Ok(next_token) = tokens.curr(true) {
                    println!("DEBUG: After bump, next token: {:?} '{}'", next_token.key(), next_token.con());
                }
                name
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some("Expected identifier in type declaration".to_string()),
                    loc: Some(name_token.loc().clone()),
                    src: name_token.loc().source().clone(),
                }));
            }
        } else {
            return Err(catch!(Typo::ParserMissmatch {
                msg: Some("Expected identifier in type declaration".to_string()),
                loc: None,
                src: None,
            }));
        };
        
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Expect colon
        if let Ok(colon_token) = tokens.curr(true) {
            println!("DEBUG: Found token after name in type decl: {:?} with content: '{}'", colon_token.key(), colon_token.con());
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::Colon) = colon_token.key() {
                tokens.bump(); // consume ':'
                if let Err(e) = self.skip_whitespace_and_comments(tokens) {
                return Err(vec![e]);
            }
                
                // Get type definition (rec, ent, etc.)
                let type_def = if let Ok(type_token) = tokens.curr(true) {
                    if let crate::token::KEYWORD::Identifier = type_token.key() {
                        let type_name = type_token.con().clone();
                        tokens.bump();
                        match type_name.as_str() {
                            "rec" => TypeDefinition::Record { fields: std::collections::HashMap::new() },
                            "ent" => TypeDefinition::Entry { variants: std::collections::HashMap::new() },
                            _ => TypeDefinition::Alias { target: FolType::Named { name: type_name } },
                        }
                    } else {
                        return Err(catch!(Typo::ParserMissmatch {
                            msg: Some("Expected type after colon in type declaration".to_string()),
                            loc: Some(type_token.loc().clone()),
                            src: type_token.loc().source().clone(),
                        }));
                    }
                } else {
                    return Err(catch!(Typo::ParserMissmatch {
                        msg: Some("Expected type after colon in type declaration".to_string()),
                        loc: None,
                        src: None,
                    }));
                };
                
                // Skip = { definition } for now
                if let Err(e) = self.skip_to_next_declaration(tokens) {
                    return Err(vec![e]);
                }
                
                Ok(AstNode::TypeDecl {
                    options: vec![],
                    generics: vec![],
                    name,
                    type_def,
                })
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some("Expected ':' after name in type declaration".to_string()),
                    loc: Some(colon_token.loc().clone()),
                    src: colon_token.loc().source().clone(),
                }));
            }
        } else {
            return Err(catch!(Typo::ParserMissmatch {
                msg: Some("Expected ':' after name in type declaration".to_string()),
                loc: None,
                src: None,
            }));
        }
    }
    
    /// Check if current token starts a const declaration: con
    fn is_const_declaration(&self, tokens: &lexer::Elements) -> bool {
        if let Ok(current) = tokens.curr(true) {
            matches!(current.key(), crate::token::KEYWORD::Keyword(crate::token::buildin::BUILDIN::Con))
        } else {
            false
        }
    }
    
    /// Parse con declaration: con name: type = value
    fn parse_const_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        tokens.bump(); // consume 'con'
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Get name
        let name = if let Ok(name_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Identifier = name_token.key() {
                let name = name_token.con().clone();
                tokens.bump();
                name
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some("Expected identifier in const declaration".to_string()),
                    loc: Some(name_token.loc().clone()),
                    src: name_token.loc().source().clone(),
                }));
            }
        } else {
            return Err(catch!(Typo::ParserMissmatch {
                msg: Some("Expected identifier in const declaration".to_string()),
                loc: None,
                src: None,
            }));
        };
        
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Expect colon
        let type_hint = if let Ok(colon_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::Colon) = colon_token.key() {
                tokens.bump(); // consume ':'
                if let Err(e) = self.skip_whitespace_and_comments(tokens) {
                return Err(vec![e]);
            }
                
                // Get type
                if let Ok(type_token) = tokens.curr(true) {
                    if let crate::token::KEYWORD::Identifier = type_token.key() {
                        let type_name = type_token.con().clone();
                        tokens.bump();
                        Some(FolType::Named { name: type_name })
                    } else {
                        Some(FolType::Named { name: "unknown".to_string() })
                    }
                } else {
                    Some(FolType::Named { name: "unknown".to_string() })
                }
            } else {
                None
            }
        } else {
            None
        };
        
        // Skip = value for now
        self.skip_to_next_declaration(tokens);
        
        // For now, treat constants as immutable variables
        Ok(AstNode::VarDecl {
            options: vec![],
            name,
            type_hint,
            value: None,
        })
    }
    
    /// Check if current token starts an alias declaration: ali
    fn is_alias_declaration(&self, tokens: &lexer::Elements) -> bool {
        if let Ok(current) = tokens.curr(true) {
            matches!(current.key(), crate::token::KEYWORD::Keyword(crate::token::buildin::BUILDIN::Ali))
        } else {
            false
        }
    }
    
    /// Parse ali declaration: ali name: target_type
    fn parse_alias_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        tokens.bump(); // consume 'ali'
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Get name
        let name = if let Ok(name_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Identifier = name_token.key() {
                let name = name_token.con().clone();
                tokens.bump();
                name
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some("Expected identifier in alias declaration".to_string()),
                    loc: Some(name_token.loc().clone()),
                    src: name_token.loc().source().clone(),
                }));
            }
        } else {
            return Err(catch!(Typo::ParserMissmatch {
                msg: Some("Expected identifier in alias declaration".to_string()),
                loc: None,
                src: None,
            }));
        };
        
        if let Err(e) = self.skip_whitespace_and_comments(tokens) {
            return Err(vec![e]);
        }
        
        // Expect colon
        let target = if let Ok(colon_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::Colon) = colon_token.key() {
                tokens.bump(); // consume ':'
                if let Err(e) = self.skip_whitespace_and_comments(tokens) {
                return Err(vec![e]);
            }
                
                // Get target type
                if let Ok(type_token) = tokens.curr(true) {
                    if let crate::token::KEYWORD::Identifier = type_token.key() {
                        let type_name = type_token.con().clone();
                        tokens.bump();
                        FolType::Named { name: type_name }
                    } else {
                        FolType::Named { name: "unknown".to_string() }
                    }
                } else {
                    FolType::Named { name: "unknown".to_string() }
                }
            } else {
                FolType::Named { name: "unknown".to_string() }
            }
        } else {
            FolType::Named { name: "unknown".to_string() }
        };
        
        // Skip rest for now
        self.skip_to_next_declaration(tokens);
        
        Ok(AstNode::AliasDecl {
            name,
            target,
        })
    }
    
    /// Check if current token starts an impl declaration: imp
    fn is_impl_declaration(&self, tokens: &lexer::Elements) -> bool {
        if let Ok(current) = tokens.curr(true) {
            matches!(current.key(), crate::token::KEYWORD::Keyword(crate::token::buildin::BUILDIN::Imp))
        } else {
            false
        }
    }
    
    /// Parse imp declaration - treat as type declaration for now
    fn parse_impl_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        tokens.bump(); // consume 'imp'
        
        // Skip implementation for now
        self.skip_to_next_declaration(tokens);
        
        // Return a placeholder type declaration
        Ok(AstNode::TypeDecl {
            options: vec![],
            generics: vec![],
            name: "impl_placeholder".to_string(),
            type_def: TypeDefinition::Alias { target: FolType::Named { name: "unknown".to_string() } },
        })
    }
    
    /// Check if current token starts a segment declaration: seg
    fn is_segment_declaration(&self, tokens: &lexer::Elements) -> bool {
        if let Ok(current) = tokens.curr(true) {
            matches!(current.key(), crate::token::KEYWORD::Keyword(crate::token::buildin::BUILDIN::Seg))
        } else {
            false
        }
    }
    
    /// Parse seg declaration
    fn parse_segment_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<AstNode, Box<dyn Glitch>> {
        tokens.bump(); // consume 'seg'
        
        // Get name
        let name = if let Ok(name_token) = tokens.curr(true) {
            if let crate::token::KEYWORD::Identifier = name_token.key() {
                let name = name_token.con().clone();
                tokens.bump();
                name
            } else {
                return Err(catch!(Typo::ParserMissmatch {
                    msg: Some("Expected identifier after seg".to_string()),
                    loc: Some(name_token.loc().clone()),
                    src: name_token.loc().source().clone(),
                }));
            }
        } else {
            return Err(catch!(Typo::ParserMissmatch {
                msg: Some("Expected identifier after seg".to_string()),
                loc: None,
                src: None,
            }));
        };
        
        // Skip rest for now
        self.skip_to_next_declaration(tokens);
        
        // Treat segment as a type for now
        Ok(AstNode::TypeDecl {
            options: vec![],
            generics: vec![],
            name,
            type_def: TypeDefinition::Alias { target: FolType::Module { name: "segment".to_string() } },
        })
    }
    
    /// Skip tokens until we find the next top-level declaration
    fn skip_to_next_declaration(&mut self, tokens: &mut lexer::Elements) -> Result<(), Box<dyn Glitch>> {
        let mut brace_depth = 0;
        let mut paren_depth = 0;
        
        while let Ok(current) = tokens.curr(true) {
            match current.key() {
                // Track braces and parens to skip properly
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::CurlyO) => {
                    brace_depth += 1;
                    tokens.bump();
                }
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::CurlyC) => {
                    if brace_depth > 0 {
                        brace_depth -= 1;
                    }
                    tokens.bump();
                    if brace_depth == 0 && paren_depth == 0 {
                        break; // End of declaration
                    }
                }
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::RoundO) => {
                    paren_depth += 1;
                    tokens.bump();
                }
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::RoundC) => {
                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }
                    tokens.bump();
                }
                // Check for next top-level keyword
                crate::token::KEYWORD::Keyword(keyword) if brace_depth == 0 && paren_depth == 0 => {
                    match keyword {
                        crate::token::buildin::BUILDIN::Use |
                        crate::token::buildin::BUILDIN::Typ |
                        crate::token::buildin::BUILDIN::Con |
                        crate::token::buildin::BUILDIN::Fun |
                        crate::token::buildin::BUILDIN::Pro |
                        crate::token::buildin::BUILDIN::Ali |
                        crate::token::buildin::BUILDIN::Imp |
                        crate::token::buildin::BUILDIN::Seg => {
                            break; // Found next declaration
                        }
                        _ => { tokens.bump(); }
                    }
                }
                // Check for ~ (tilde) prefix
                crate::token::KEYWORD::Symbol(crate::token::symbol::SYMBOL::Home) if brace_depth == 0 && paren_depth == 0 => {
                    break; // Found ~typ declaration
                }
                // End of file
                crate::token::KEYWORD::Void(crate::token::void::VOID::EndFile) => {
                    break;
                }
                _ => {
                    tokens.bump();
                }
            }
        }
        Ok(())
    }
}
