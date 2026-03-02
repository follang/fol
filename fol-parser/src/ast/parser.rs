// AST Parser Implementation for FOL

use super::{
    AstNode, BinaryOperator, FolType, FunOption, Generic, Literal, LoopCondition, Parameter,
    UnaryOperator, UseOption, VarOption, WhenCase,
};
use fol_lexer::token::{BUILDIN, KEYWORD, LITERAL, OPERATOR, SYMBOL};
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

        for _ in 0..8_192 {
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

            if matches!(key, KEYWORD::Keyword(BUILDIN::Var)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_var_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }

                if let Ok(current) = tokens.curr(false) {
                    let after = (
                        current.loc().row(),
                        current.loc().col(),
                        current.con().to_string(),
                    );
                    if before == after && tokens.bump().is_none() {
                        break;
                    }
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Use)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_use_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Fun)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_fun_decl(tokens) {
                    Ok(node) => {
                        if let AstNode::FunDecl { body, .. } = &node {
                            declarations.extend(body.clone());
                        }
                        declarations.push(node);
                    }
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Pro)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_pro_decl(tokens) {
                    Ok(node) => {
                        if let AstNode::ProDecl { body, .. } = &node {
                            declarations.extend(body.clone());
                        }
                        declarations.push(node);
                    }
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Return)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_return_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if key.is_ident() && self.lookahead_is_call(tokens) && self.can_start_assignment(tokens)
            {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_call_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Break)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_break_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Yeild)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_yield_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(
                key,
                KEYWORD::Keyword(BUILDIN::Panic)
                    | KEYWORD::Keyword(BUILDIN::Report)
                    | KEYWORD::Keyword(BUILDIN::Check)
                    | KEYWORD::Keyword(BUILDIN::Assert)
            ) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_builtin_call_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::When)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_when_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::If)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_if_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Loop)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_loop_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if key.is_ident()
                && self.lookahead_is_assignment(tokens)
                && self.can_start_assignment(tokens)
            {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_assignment_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
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

    fn parse_var_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut type_hint = None;
        let mut value = None;

        if tokens.bump().is_none() {
            return Err(Box::new(ParseError {
                message: "Unexpected EOF after 'var' declaration".to_string(),
                file: None,
                line: 1,
                column: 1,
                length: 1,
            }));
        }
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = if name_token.key().is_ident() {
            let parsed_name = name_token.con().trim().to_string();
            let _ = tokens.bump();
            parsed_name
        } else {
            return Err(Box::new(ParseError::from_token(
                &name_token,
                "Expected identifier after 'var'".to_string(),
            )));
        };

        self.skip_ignorable(tokens);

        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                let hint_token = tokens.curr(false)?;
                let hint_name = hint_token.con().trim().to_string();
                if hint_token.key().is_ident() || hint_token.key().is_buildin() {
                    type_hint = Some(FolType::Named { name: hint_name });
                    let _ = tokens.bump();
                } else {
                    return Err(Box::new(ParseError::from_token(
                        &hint_token,
                        "Expected type hint after ':' in var declaration".to_string(),
                    )));
                }
            }
        }

        self.skip_ignorable(tokens);

        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                let parsed_value = self.parse_logical_expression(tokens)?;
                value = Some(Box::new(parsed_value));
            }
        }

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::VarDecl {
            options: vec![VarOption::Normal],
            name,
            type_hint,
            value,
        })
    }

    fn parse_use_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let use_token = tokens.curr(false)?;
        if !matches!(use_token.key(), KEYWORD::Keyword(BUILDIN::Use)) {
            return Err(Box::new(ParseError::from_token(
                &use_token,
                "Expected 'use' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        if !name_token.key().is_ident() {
            return Err(Box::new(ParseError::from_token(
                &name_token,
                "Expected use declaration name".to_string(),
            )));
        }
        let name = name_token.con().trim().to_string();
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' after use name".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let type_token = tokens.curr(false)?;
        let path_type = self.parse_type_reference(&type_token)?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' in use declaration".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start use path".to_string(),
            )));
        }
        let _ = tokens.bump();

        let path = self.parse_use_path(tokens)?;

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::UseDecl {
            options: Vec::<UseOption>::new(),
            name,
            path_type,
            path,
        })
    }

    fn parse_use_path(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<String, Box<dyn Glitch>> {
        let mut path = String::new();

        for _ in 0..512 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(path);
            }

            let segment = token.con().trim();
            if !segment.is_empty() {
                path.push_str(segment);
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        Err(Box::new(ParseError {
            message: "Use path parsing exceeded safety bound".to_string(),
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }))
    }

    fn parse_fun_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let fun_token = tokens.curr(false)?;
        if !matches!(fun_token.key(), KEYWORD::Keyword(BUILDIN::Fun)) {
            return Err(Box::new(ParseError::from_token(
                &fun_token,
                "Expected 'fun' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        if !name_token.key().is_ident() {
            return Err(Box::new(ParseError::from_token(
                &name_token,
                "Expected function name after 'fun'".to_string(),
            )));
        }
        let name = name_token.con().trim().to_string();
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_paren = tokens.curr(false)?;
        if !matches!(open_paren.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_paren,
                "Expected '(' after function name".to_string(),
            )));
        }
        let _ = tokens.bump();

        let params = self.parse_parameter_list(tokens)?;

        self.skip_ignorable(tokens);
        let mut return_type = None;
        let mut error_type = None;
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                let typ_token = tokens.curr(false)?;
                return_type = Some(self.parse_type_reference(&typ_token)?);
                let _ = tokens.bump();

                self.skip_ignorable(tokens);
                if let Ok(err_sep) = tokens.curr(false) {
                    if matches!(err_sep.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        let err_type_token = tokens.curr(false)?;
                        error_type = Some(self.parse_type_reference(&err_type_token)?);
                        let _ = tokens.bump();
                    }
                }
            }
        }

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' before function body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start function body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let body = self.parse_block_body(tokens)?;

        Ok(AstNode::FunDecl {
            options: vec![FunOption::Mutable],
            generics: Vec::<Generic>::new(),
            name,
            params,
            return_type,
            error_type,
            body,
        })
    }

    fn parse_pro_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let pro_token = tokens.curr(false)?;
        if !matches!(pro_token.key(), KEYWORD::Keyword(BUILDIN::Pro)) {
            return Err(Box::new(ParseError::from_token(
                &pro_token,
                "Expected 'pro' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        if !name_token.key().is_ident() {
            return Err(Box::new(ParseError::from_token(
                &name_token,
                "Expected procedure name after 'pro'".to_string(),
            )));
        }
        let name = name_token.con().trim().to_string();
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_paren = tokens.curr(false)?;
        if !matches!(open_paren.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_paren,
                "Expected '(' after procedure name".to_string(),
            )));
        }
        let _ = tokens.bump();

        let params = self.parse_parameter_list(tokens)?;

        self.skip_ignorable(tokens);
        let mut return_type = None;
        let mut error_type = None;
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                let typ_token = tokens.curr(false)?;
                return_type = Some(self.parse_type_reference(&typ_token)?);
                let _ = tokens.bump();

                self.skip_ignorable(tokens);
                if let Ok(err_sep) = tokens.curr(false) {
                    if matches!(err_sep.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        let err_type_token = tokens.curr(false)?;
                        error_type = Some(self.parse_type_reference(&err_type_token)?);
                        let _ = tokens.bump();
                    }
                }
            }
        }

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' before procedure body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start procedure body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let body = self.parse_block_body(tokens)?;

        Ok(AstNode::ProDecl {
            options: vec![FunOption::Mutable],
            generics: Vec::<Generic>::new(),
            name,
            params,
            return_type,
            error_type,
            body,
        })
    }

    fn parse_parameter_list(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<Parameter>, Box<dyn Glitch>> {
        let mut params = Vec::new();

        for _ in 0..512 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok(params);
            }

            if !token.key().is_ident() {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Expected parameter name".to_string(),
                )));
            }

            let param_name = token.con().trim().to_string();
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let colon = tokens.curr(false)?;
            if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                return Err(Box::new(ParseError::from_token(
                    &colon,
                    "Expected ':' after parameter name".to_string(),
                )));
            }
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let type_token = tokens.curr(false)?;
            let param_type = self.parse_type_reference(&type_token)?;
            let _ = tokens.bump();

            params.push(Parameter {
                name: param_name.clone(),
                param_type,
                is_borrowable: param_name.chars().all(|ch| {
                    !ch.is_ascii_lowercase() && (ch.is_ascii_alphanumeric() || ch == '_')
                }),
                default: None,
            });

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok(params);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or ')' after parameter".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Parameter parsing exceeded safety bound".to_string(),
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }))
    }

    fn parse_type_reference(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<FolType, Box<dyn Glitch>> {
        if token.key().is_ident() || token.key().is_buildin() {
            return Ok(FolType::Named {
                name: token.con().trim().to_string(),
            });
        }

        Err(Box::new(ParseError::from_token(
            token,
            "Expected type reference".to_string(),
        )))
    }

    fn parse_block_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();

        for _ in 0..8_192 {
            self.skip_ignorable(tokens);

            let token = tokens.curr(false)?;
            let key = token.key();

            if matches!(key, KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if key.is_eof() {
                return Ok(body);
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Return)) {
                body.push(self.parse_return_stmt(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Break)) {
                body.push(self.parse_break_stmt(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Yeild)) {
                body.push(self.parse_yield_stmt(tokens)?);
                continue;
            }

            if matches!(
                key,
                KEYWORD::Keyword(BUILDIN::Panic)
                    | KEYWORD::Keyword(BUILDIN::Report)
                    | KEYWORD::Keyword(BUILDIN::Check)
                    | KEYWORD::Keyword(BUILDIN::Assert)
            ) {
                body.push(self.parse_builtin_call_stmt(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Var)) {
                body.push(self.parse_var_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::When)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                body.push(self.parse_when_stmt(tokens)?);
                self.bump_if_no_progress(tokens, before);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::If)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                body.push(self.parse_if_stmt(tokens)?);
                self.bump_if_no_progress(tokens, before);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Loop)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                body.push(self.parse_loop_stmt(tokens)?);
                self.bump_if_no_progress(tokens, before);
                continue;
            }

            if key.is_ident()
                && self.lookahead_is_assignment(tokens)
                && self.can_start_assignment(tokens)
            {
                body.push(self.parse_assignment_stmt(tokens)?);
                continue;
            }

            if key.is_ident()
                && (self.lookahead_is_call(tokens) || self.lookahead_is_method_call(tokens))
                && self.can_start_assignment(tokens)
            {
                body.push(self.parse_call_stmt(tokens)?);
                continue;
            }

            if key.is_ident() {
                body.push(AstNode::Identifier {
                    name: token.con().trim().to_string(),
                });
            } else if key.is_literal() {
                body.push(self.parse_lexer_literal(&token)?);
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        Ok(body)
    }

    fn parse_return_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        if tokens.bump().is_none() {
            return Ok(AstNode::Return { value: None });
        }

        self.skip_ignorable(tokens);

        let value = match tokens.curr(false) {
            Ok(token) if token.key().is_terminal() => None,
            Ok(_) => Some(Box::new(self.parse_logical_expression(tokens)?)),
            Err(_) => None,
        };

        for _ in 0..64 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if token.key().is_terminal() {
                let _ = tokens.bump();
                break;
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        Ok(AstNode::Return { value })
    }

    fn parse_break_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let break_token = tokens.curr(false)?;
        if !matches!(break_token.key(), KEYWORD::Keyword(BUILDIN::Break)) {
            return Err(Box::new(ParseError::from_token(
                &break_token,
                "Expected 'break' statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.consume_optional_semicolon(tokens);

        Ok(AstNode::Break)
    }

    fn parse_yield_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let yield_token = tokens.curr(false)?;
        if !matches!(yield_token.key(), KEYWORD::Keyword(BUILDIN::Yeild)) {
            return Err(Box::new(ParseError::from_token(
                &yield_token,
                "Expected 'yield' statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let value = self.parse_logical_expression(tokens)?;

        for _ in 0..64 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if token.key().is_terminal() {
                let _ = tokens.bump();
                break;
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        Ok(AstNode::Yield {
            value: Box::new(value),
        })
    }

    fn parse_builtin_call_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let keyword_token = tokens.curr(false)?;
        let name = match keyword_token.key() {
            KEYWORD::Keyword(BUILDIN::Panic) => "panic",
            KEYWORD::Keyword(BUILDIN::Report) => "report",
            KEYWORD::Keyword(BUILDIN::Check) => "check",
            KEYWORD::Keyword(BUILDIN::Assert) => "assert",
            _ => {
                return Err(Box::new(ParseError::from_token(
                    &keyword_token,
                    "Expected builtin diagnostic statement".to_string(),
                )));
            }
        }
        .to_string();

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let mut args = Vec::new();
        if let Ok(token) = tokens.curr(false) {
            if !token.key().is_terminal() {
                let expr = self.parse_logical_expression(tokens)?;
                args.push(expr);
            }
        }

        for _ in 0..64 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if token.key().is_terminal() {
                let _ = tokens.bump();
                break;
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        Ok(AstNode::FunctionCall { name, args })
    }

    fn parse_when_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let when_token = tokens.curr(false)?;
        if !matches!(when_token.key(), KEYWORD::Keyword(BUILDIN::When)) {
            return Err(Box::new(ParseError::from_token(
                &when_token,
                "Expected 'when' statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let open_expr = tokens.curr(false)?;
        if !matches!(open_expr.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_expr,
                "Expected '(' after 'when'".to_string(),
            )));
        }
        let _ = tokens.bump();

        let expr = self.parse_logical_expression(tokens)?;
        self.skip_ignorable(tokens);

        let close_expr = tokens.curr(false)?;
        if !matches!(close_expr.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
            return Err(Box::new(ParseError::from_token(
                &close_expr,
                "Expected ')' after when expression".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_cases = tokens.curr(false)?;
        if !matches!(open_cases.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_cases,
                "Expected '{' to start when cases".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut cases = Vec::new();
        let mut default = None;

        for _ in 0..1024 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Case)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                let open_cond = tokens.curr(false)?;
                if !matches!(open_cond.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                    return Err(Box::new(ParseError::from_token(
                        &open_cond,
                        "Expected '(' after case".to_string(),
                    )));
                }
                let _ = tokens.bump();

                let condition = self.parse_logical_expression(tokens)?;
                self.skip_ignorable(tokens);
                let close_cond = tokens.curr(false)?;
                if !matches!(close_cond.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                    return Err(Box::new(ParseError::from_token(
                        &close_cond,
                        "Expected ')' after case condition".to_string(),
                    )));
                }
                let _ = tokens.bump();

                self.skip_ignorable(tokens);
                let body = self.parse_case_body(tokens)?;
                cases.push(WhenCase::Case { condition, body });
                continue;
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                let body = self.parse_case_body(tokens)?;
                default = Some(body);
                continue;
            }

            let _ = tokens.bump();
        }

        Ok(AstNode::When {
            expr: Box::new(expr),
            cases,
            default,
        })
    }

    fn parse_if_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let if_token = tokens.curr(false)?;
        if !matches!(if_token.key(), KEYWORD::Keyword(BUILDIN::If)) {
            return Err(Box::new(ParseError::from_token(
                &if_token,
                "Expected 'if' statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let open_cond = tokens.curr(false)?;
        if !matches!(open_cond.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_cond,
                "Expected '(' after 'if'".to_string(),
            )));
        }
        let _ = tokens.bump();

        let condition = self.parse_logical_expression(tokens)?;
        self.skip_ignorable(tokens);

        let close_cond = tokens.curr(false)?;
        if !matches!(close_cond.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
            return Err(Box::new(ParseError::from_token(
                &close_cond,
                "Expected ')' after if condition".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let then_body = self.parse_case_body(tokens)?;

        self.skip_ignorable(tokens);
        let else_body = if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Else)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                let else_target = tokens.curr(false)?;
                if matches!(else_target.key(), KEYWORD::Keyword(BUILDIN::If)) {
                    Some(vec![self.parse_if_stmt(tokens)?])
                } else if matches!(else_target.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                    Some(self.parse_case_body(tokens)?)
                } else {
                    return Err(Box::new(ParseError::from_token(
                        &else_target,
                        "Expected 'if' or '{' after else".to_string(),
                    )));
                }
            } else if matches!(token.key(), KEYWORD::Keyword(BUILDIN::If)) {
                Some(vec![self.parse_if_stmt(tokens)?])
            } else if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                Some(self.parse_case_body(tokens)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(AstNode::When {
            expr: Box::new(condition.clone()),
            cases: vec![WhenCase::Case {
                condition,
                body: then_body,
            }],
            default: else_body,
        })
    }

    fn parse_case_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start case/default body".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.parse_block_body(tokens)
    }

    fn parse_loop_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let loop_token = tokens.curr(false)?;
        if !matches!(loop_token.key(), KEYWORD::Keyword(BUILDIN::Loop)) {
            return Err(Box::new(ParseError::from_token(
                &loop_token,
                "Expected 'loop' statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let open_cond = tokens.curr(false)?;
        if !matches!(open_cond.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_cond,
                "Expected '(' after 'loop'".to_string(),
            )));
        }
        let _ = tokens.bump();

        let condition = self.parse_loop_condition(tokens)?;
        self.skip_ignorable(tokens);

        let close_cond = tokens.curr(false)?;
        if !matches!(close_cond.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
            return Err(Box::new(ParseError::from_token(
                &close_cond,
                "Expected ')' after loop condition".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start loop body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let body = self.parse_block_body(tokens)?;

        Ok(AstNode::Loop {
            condition: Box::new(condition),
            body,
        })
    }

    fn parse_loop_condition(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<LoopCondition, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);

        let current = tokens.curr(false)?;
        if current.key().is_ident()
            && matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Keyword(BUILDIN::In))
            )
        {
            let var = current.con().trim().to_string();
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let in_token = tokens.curr(false)?;
            if !matches!(in_token.key(), KEYWORD::Keyword(BUILDIN::In)) {
                return Err(Box::new(ParseError::from_token(
                    &in_token,
                    "Expected 'in' in loop iteration condition".to_string(),
                )));
            }
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let iterable = self.parse_logical_expression(tokens)?;
            self.skip_ignorable(tokens);

            let condition = if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Keyword(BUILDIN::When)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    Some(Box::new(self.parse_logical_expression(tokens)?))
                } else {
                    None
                }
            } else {
                None
            };

            return Ok(LoopCondition::Iteration {
                var,
                iterable: Box::new(iterable),
                condition,
            });
        }

        let condition_expr = self.parse_logical_expression(tokens)?;
        Ok(LoopCondition::Condition(Box::new(condition_expr)))
    }

    fn parse_assignment_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let target_token = tokens.curr(false)?;
        let target = AstNode::Identifier {
            name: target_token.con().trim().to_string(),
        };

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let assign_token = tokens.curr(false)?;
        let mut compound_op = self.compound_assignment_op(&assign_token.key());
        let mut is_simple_assign = matches!(assign_token.key(), KEYWORD::Symbol(SYMBOL::Equal));

        if matches!(assign_token.key(), KEYWORD::Symbol(SYMBOL::Percent)) {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            let eq_token = tokens.curr(false)?;
            if matches!(eq_token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                compound_op = Some(BinaryOperator::Mod);
                is_simple_assign = false;
            } else {
                return Err(Box::new(ParseError::from_token(
                    &eq_token,
                    "Expected '=' after '%' in compound assignment".to_string(),
                )));
            }
        }

        if !is_simple_assign && compound_op.is_none() {
            return Err(Box::new(ParseError::from_token(
                &assign_token,
                "Expected assignment operator in assignment statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let parsed_value = self.parse_logical_expression(tokens)?;
        let value = if let Some(op) = compound_op {
            AstNode::BinaryOp {
                op,
                left: Box::new(target.clone()),
                right: Box::new(parsed_value),
            }
        } else {
            parsed_value
        };

        for _ in 0..64 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if token.key().is_terminal() {
                let _ = tokens.bump();
                break;
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        Ok(AstNode::Assignment {
            target: Box::new(target),
            value: Box::new(value),
        })
    }

    fn parse_call_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let call = if self.lookahead_is_method_call(tokens) {
            self.parse_method_call_expr(tokens)?
        } else {
            self.parse_call_expr(tokens)?
        };

        self.consume_optional_semicolon(tokens);

        Ok(call)
    }

    fn parse_call_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let name_token = tokens.curr(false)?;
        if !name_token.key().is_ident() {
            return Err(Box::new(ParseError::from_token(
                &name_token,
                "Expected identifier for function call".to_string(),
            )));
        }
        let name = name_token.con().trim().to_string();
        let _ = tokens.bump();
        let args =
            self.parse_open_paren_and_call_args(tokens, "Expected '(' after function name")?;

        Ok(AstNode::FunctionCall { name, args })
    }

    fn parse_method_call_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let object_token = tokens.curr(false)?;
        if !object_token.key().is_ident() {
            return Err(Box::new(ParseError::from_token(
                &object_token,
                "Expected object identifier for method call".to_string(),
            )));
        }

        let object = AstNode::Identifier {
            name: object_token.con().trim().to_string(),
        };
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let dot = tokens.curr(false)?;
        if !matches!(dot.key(), KEYWORD::Symbol(SYMBOL::Dot)) {
            return Err(Box::new(ParseError::from_token(
                &dot,
                "Expected '.' after object identifier".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let method_token = tokens.curr(false)?;
        if !method_token.key().is_ident() {
            return Err(Box::new(ParseError::from_token(
                &method_token,
                "Expected method name after '.'".to_string(),
            )));
        }
        let method = method_token.con().trim().to_string();
        let _ = tokens.bump();
        let args = self.parse_open_paren_and_call_args(tokens, "Expected '(' after method name")?;

        Ok(AstNode::MethodCall {
            object: Box::new(object),
            method,
            args,
        })
    }

    fn parse_open_paren_and_call_args(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        expected_open_error: &str,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);

        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                expected_open_error.to_string(),
            )));
        }

        let _ = tokens.bump();
        self.parse_call_args(tokens)
    }

    fn parse_call_args(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut args = Vec::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                break;
            }

            args.push(self.parse_logical_expression(tokens)?);
            self.skip_ignorable(tokens);

            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                break;
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or ')' in call arguments".to_string(),
            )));
        }

        Ok(args)
    }

    fn lookahead_is_assignment(&self, tokens: &fol_lexer::lexer::stage3::Elements) -> bool {
        let mut found_percent = false;
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if found_percent {
                return matches!(key, KEYWORD::Symbol(SYMBOL::Equal));
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::Percent)) {
                found_percent = true;
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::Equal))
                || self.compound_assignment_op(&key).is_some();
        }

        false
    }

    fn lookahead_is_call(&self, tokens: &fol_lexer::lexer::stage3::Elements) -> bool {
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::RoundO));
        }

        false
    }

    fn lookahead_is_method_call(&self, tokens: &fol_lexer::lexer::stage3::Elements) -> bool {
        let mut saw_dot = false;
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if !saw_dot {
                if matches!(key, KEYWORD::Symbol(SYMBOL::Dot)) {
                    saw_dot = true;
                    continue;
                }
                return false;
            }

            if key.is_ident() {
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::RoundO));
        }

        false
    }

    fn can_start_assignment(&self, tokens: &fol_lexer::lexer::stage3::Elements) -> bool {
        match self.previous_significant_key(tokens) {
            None => true,
            Some(KEYWORD::Symbol(SYMBOL::CurlyO)) => true,
            Some(KEYWORD::Symbol(SYMBOL::Semi)) => true,
            Some(key) if key.is_terminal() => true,
            _ => false,
        }
    }

    fn previous_significant_key(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> Option<KEYWORD> {
        for candidate in tokens.prev_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            return Some(key);
        }

        None
    }

    fn bump_if_no_progress(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        before: (usize, usize, String),
    ) {
        if let Ok(current) = tokens.curr(false) {
            let after = (
                current.loc().row(),
                current.loc().col(),
                current.con().to_string(),
            );
            if before == after {
                let _ = tokens.bump();
            }
        }
    }

    fn token_is_word(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
        word: &str,
    ) -> bool {
        token.con().trim() == word
            || matches!(
                (token.key(), word),
                (KEYWORD::Keyword(BUILDIN::And), "and")
                    | (KEYWORD::Keyword(BUILDIN::Or), "or")
                    | (KEYWORD::Keyword(BUILDIN::Xor), "xor")
                    | (KEYWORD::Keyword(BUILDIN::Nand), "nand")
                    | (KEYWORD::Keyword(BUILDIN::Nor), "nor")
                    | (KEYWORD::Keyword(BUILDIN::Not), "not")
            )
    }

    fn compound_assignment_op(&self, key: &KEYWORD) -> Option<BinaryOperator> {
        match key {
            KEYWORD::Operator(OPERATOR::Addeq) => Some(BinaryOperator::Add),
            KEYWORD::Operator(OPERATOR::Subeq) => Some(BinaryOperator::Sub),
            KEYWORD::Operator(OPERATOR::Multeq) => Some(BinaryOperator::Mul),
            KEYWORD::Operator(OPERATOR::Diveq) => Some(BinaryOperator::Div),
            _ => None,
        }
    }

    fn parse_logical_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.parse_logical_or_expression(tokens)
    }

    fn parse_logical_or_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_logical_xor_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            if self.token_is_word(&op_token, "or") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_logical_xor_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op: BinaryOperator::Or,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            if self.token_is_word(&op_token, "nor") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_logical_xor_expression(tokens)?;
                lhs = AstNode::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(AstNode::BinaryOp {
                        op: BinaryOperator::Or,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_logical_xor_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_logical_and_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            if self.token_is_word(&op_token, "xor") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_logical_and_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op: BinaryOperator::Xor,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_logical_and_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_comparison_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            if self.token_is_word(&op_token, "and") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_comparison_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op: BinaryOperator::And,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            if self.token_is_word(&op_token, "nand") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_comparison_expression(tokens)?;
                lhs = AstNode::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(AstNode::BinaryOp {
                        op: BinaryOperator::And,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_comparison_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_add_sub_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            let next_key = self.next_significant_key_from_window(tokens);
            let op_text = op_token.con().trim().to_string();
            let (binary_op, consume_count) = match op_token.key() {
                KEYWORD::Operator(OPERATOR::Equal) => (Some(BinaryOperator::Eq), 1),
                KEYWORD::Operator(OPERATOR::Noteq) => (Some(BinaryOperator::Ne), 1),
                KEYWORD::Operator(OPERATOR::Greateq) => (Some(BinaryOperator::Ge), 1),
                KEYWORD::Operator(OPERATOR::Lesseq) => (Some(BinaryOperator::Le), 1),
                KEYWORD::Symbol(SYMBOL::AngleC) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Ge), 2)
                    } else {
                        (Some(BinaryOperator::Gt), 1)
                    }
                }
                KEYWORD::Symbol(SYMBOL::AngleO) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Le), 2)
                    } else {
                        (Some(BinaryOperator::Lt), 1)
                    }
                }
                KEYWORD::Symbol(SYMBOL::Equal) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Eq), 2)
                    } else {
                        (None, 0)
                    }
                }
                KEYWORD::Symbol(SYMBOL::Bang) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Ne), 2)
                    } else {
                        (None, 0)
                    }
                }
                _ => match op_text.as_str() {
                    ">=" => (Some(BinaryOperator::Ge), 1),
                    "<=" => (Some(BinaryOperator::Le), 1),
                    "==" => (Some(BinaryOperator::Eq), 1),
                    "!=" => (Some(BinaryOperator::Ne), 1),
                    ">" => (Some(BinaryOperator::Gt), 1),
                    "<" => (Some(BinaryOperator::Lt), 1),
                    _ => (None, 0),
                },
            };

            if let Some(op) = binary_op {
                for _ in 0..consume_count {
                    self.consume_significant_token(tokens);
                }
                let rhs = self.parse_add_sub_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn next_significant_key_from_window(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> Option<KEYWORD> {
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            return Some(key);
        }

        None
    }

    fn consume_significant_token(&self, tokens: &mut fol_lexer::lexer::stage3::Elements) {
        for _ in 0..16 {
            if tokens.bump().is_none() {
                break;
            }

            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if !(token.key().is_void() || token.key().is_comment()) {
                break;
            }
        }
    }

    fn parse_add_sub_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_mul_div_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            let binary_op = match op_token.key() {
                KEYWORD::Operator(OPERATOR::Add) | KEYWORD::Symbol(SYMBOL::Plus) => {
                    Some(BinaryOperator::Add)
                }
                KEYWORD::Operator(OPERATOR::Abstract) | KEYWORD::Symbol(SYMBOL::Minus) => {
                    Some(BinaryOperator::Sub)
                }
                _ => None,
            };

            if let Some(op) = binary_op {
                let _ = tokens.bump();
                let rhs = self.parse_mul_div_expression(tokens)?;

                lhs = AstNode::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_mul_div_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_primary_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            let binary_op = match op_token.key() {
                KEYWORD::Operator(OPERATOR::Multiply) | KEYWORD::Symbol(SYMBOL::Star) => {
                    Some(BinaryOperator::Mul)
                }
                KEYWORD::Operator(OPERATOR::Divide) | KEYWORD::Symbol(SYMBOL::Root) => {
                    Some(BinaryOperator::Div)
                }
                KEYWORD::Symbol(SYMBOL::Percent) => Some(BinaryOperator::Mod),
                _ => None,
            };

            if let Some(op) = binary_op {
                let _ = tokens.bump();
                let rhs = self.parse_primary_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    fn parse_primary_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let token = tokens.curr(false)?;

        if let Some((message, unary_op)) = self.unary_prefix_info(&token) {
            let operator_token = token.clone();
            let _ = tokens.bump();
            self.ensure_unary_operand(tokens, &operator_token, message)?;

            let operand = self.parse_primary_expression(tokens)?;
            if let Some(op) = unary_op {
                return Ok(AstNode::UnaryOp {
                    op,
                    operand: Box::new(operand),
                });
            }

            return Ok(operand);
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            let _ = tokens.bump();
            let inner = self.parse_logical_expression(tokens)?;
            self.skip_ignorable(tokens);

            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected closing ')' for parenthesized expression".to_string(),
                )));
            }

            let _ = tokens.bump();
            return Ok(inner);
        }

        if token.key().is_ident() && self.lookahead_is_method_call(tokens) {
            return self.parse_method_call_expr(tokens);
        }

        if token.key().is_ident() && self.lookahead_is_call(tokens) {
            return self.parse_call_expr(tokens);
        }

        let node = self.parse_primary(&token)?;
        let _ = tokens.bump();
        Ok(node)
    }

    fn parse_primary(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        if token.key().is_literal() {
            return self.parse_lexer_literal(token);
        }

        if token.key().is_ident() {
            return Ok(AstNode::Identifier {
                name: token.con().trim().to_string(),
            });
        }

        Err(Box::new(ParseError::from_token(
            token,
            format!("Unsupported expression token '{}'", token.con()),
        )))
    }

    fn skip_ignorable(&self, tokens: &mut fol_lexer::lexer::stage3::Elements) {
        for _ in 0..128 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if token.key().is_void() || token.key().is_comment() {
                if tokens.bump().is_none() {
                    break;
                }
                continue;
            }

            break;
        }
    }

    fn unary_prefix_info(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Option<(&'static str, Option<UnaryOperator>)> {
        if matches!(
            token.key(),
            KEYWORD::Operator(OPERATOR::Abstract) | KEYWORD::Symbol(SYMBOL::Minus)
        ) {
            return Some((
                "Expected expression after unary '-'",
                Some(UnaryOperator::Neg),
            ));
        }

        if matches!(
            token.key(),
            KEYWORD::Operator(OPERATOR::Add) | KEYWORD::Symbol(SYMBOL::Plus)
        ) {
            return Some(("Expected expression after unary '+'", None));
        }

        if self.token_is_word(token, "not") {
            return Some((
                "Expected expression after unary 'not'",
                Some(UnaryOperator::Not),
            ));
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::And)) {
            return Some((
                "Expected expression after unary '&'",
                Some(UnaryOperator::Ref),
            ));
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Star)) {
            return Some((
                "Expected expression after unary '*'",
                Some(UnaryOperator::Deref),
            ));
        }

        None
    }

    fn ensure_unary_operand(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        operator_token: &fol_lexer::lexer::stage3::element::Element,
        message: &str,
    ) -> Result<(), Box<dyn Glitch>> {
        self.skip_ignorable(tokens);

        match tokens.curr(false) {
            Ok(next) => {
                if next.key().is_void()
                    || matches!(
                        next.key(),
                        KEYWORD::Symbol(SYMBOL::Semi)
                            | KEYWORD::Symbol(SYMBOL::Comma)
                            | KEYWORD::Symbol(SYMBOL::RoundC)
                            | KEYWORD::Symbol(SYMBOL::CurlyC)
                    )
                {
                    return Err(Box::new(ParseError::from_token(&next, message.to_string())));
                }

                Ok(())
            }
            Err(_) => Err(Box::new(ParseError::from_token(
                operator_token,
                message.to_string(),
            ))),
        }
    }

    fn consume_optional_semicolon(&self, tokens: &mut fol_lexer::lexer::stage3::Elements) {
        self.skip_ignorable(tokens);
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Semi)) {
                let _ = tokens.bump();
            }
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
