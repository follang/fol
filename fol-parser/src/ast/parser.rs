// AST Parser Implementation for FOL

use super::{
    AstNode, BinaryOperator, FolType, FunOption, Generic, Literal, Parameter, UnaryOperator,
    VarOption,
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

            if matches!(key, KEYWORD::Keyword(BUILDIN::Fun)) {
                match self.parse_fun_decl(tokens) {
                    Ok(node) => {
                        if let AstNode::FunDecl { body, .. } = &node {
                            declarations.extend(body.clone());
                        }
                        declarations.push(node);
                    }
                    Err(error) => errors.push(error),
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Return)) {
                match self.parse_return_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                continue;
            }

            if key.is_ident()
                && self.lookahead_is_assignment(tokens)
                && self.can_start_assignment(tokens)
            {
                match self.parse_assignment_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
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

        self.skip_ignorable(tokens);
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Semi)) {
                let _ = tokens.bump();
            }
        }

        Ok(AstNode::VarDecl {
            options: vec![VarOption::Normal],
            name,
            type_hint,
            value,
        })
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
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                let typ_token = tokens.curr(false)?;
                return_type = Some(self.parse_type_reference(&typ_token)?);
                let _ = tokens.bump();
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

            if matches!(key, KEYWORD::Keyword(BUILDIN::Var)) {
                body.push(self.parse_var_decl(tokens)?);
                continue;
            }

            if key.is_ident()
                && self.lookahead_is_assignment(tokens)
                && self.can_start_assignment(tokens)
            {
                body.push(self.parse_assignment_stmt(tokens)?);
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
        let mut lhs = self.parse_comparison_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            let binary_op = match op_token.key() {
                KEYWORD::Keyword(BUILDIN::And) => Some(BinaryOperator::And),
                KEYWORD::Keyword(BUILDIN::Or) => Some(BinaryOperator::Or),
                _ => None,
            };

            if let Some(op) = binary_op {
                self.consume_significant_token(tokens);
                let rhs = self.parse_comparison_expression(tokens)?;
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

        if matches!(
            token.key(),
            KEYWORD::Operator(OPERATOR::Abstract) | KEYWORD::Symbol(SYMBOL::Minus)
        ) {
            let _ = tokens.bump();
            let operand = self.parse_primary_expression(tokens)?;
            return Ok(AstNode::UnaryOp {
                op: UnaryOperator::Neg,
                operand: Box::new(operand),
            });
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
