use super::*;

impl AstParser {
    fn lookahead_is_shorthand_anonymous_fun(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let mut depth = 1usize;
        let mut capture_depth = 0usize;
        let mut in_type_clause = false;
        let saw_open = true;
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            match key {
                KEYWORD::Symbol(SYMBOL::SquarO) if saw_open && depth == 0 => {
                    capture_depth += 1;
                }
                KEYWORD::Symbol(SYMBOL::SquarC) if saw_open && depth == 0 => {
                    if capture_depth == 0 {
                        return false;
                    }
                    capture_depth -= 1;
                }
                KEYWORD::Symbol(SYMBOL::RoundO) => {
                    depth += 1;
                }
                KEYWORD::Symbol(SYMBOL::RoundC) => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    if saw_open && depth == 0 {
                        continue;
                    }
                }
                KEYWORD::Symbol(SYMBOL::Colon) if saw_open && depth == 0 && capture_depth == 0 => {
                    in_type_clause = true;
                }
                KEYWORD::Symbol(SYMBOL::CurlyO) if saw_open && depth == 0 && capture_depth == 0 => {
                    return true;
                }
                _ if saw_open && depth == 0 && capture_depth == 0 && !in_type_clause => {
                    return false;
                }
                _ => {}
            }
        }

        false
    }

    pub(super) fn parse_primary_expression(
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

        let node = if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
            self.parse_pipe_lambda_expr(tokens)?
        } else if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Fun)) {
            self.parse_anonymous_fun_expr(tokens)?
        } else if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Log)) {
            self.parse_anonymous_log_expr(tokens)?
        } else if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Pro)) {
            self.parse_anonymous_pro_expr(tokens)?
        } else if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return self.parse_container_expression(tokens);
        } else if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundO))
            && self.lookahead_is_shorthand_anonymous_fun(tokens)
        {
            self.parse_shorthand_anonymous_fun_expr(tokens)?
        } else if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
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
            inner
        } else if matches!(token.key(), KEYWORD::Literal(LITERAL::Stringy))
            && matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Symbol(SYMBOL::RoundO))
            )
        {
            let name = Self::token_to_named_label(&token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &token,
                    "Expected quoted callable name".to_string(),
                )) as Box<dyn Glitch>
            })?;
            let _ = tokens.bump();
            AstNode::Identifier { name }
        } else if Self::token_can_start_path_expression(&token)
            && matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Operator(OPERATOR::Path))
            )
        {
            let name = self.parse_named_path(
                tokens,
                "Expected expression path root",
                "Expected name after '::' in expression path",
            )?;
            AstNode::Identifier { name }
        } else {
            let node = self.parse_primary(&token)?;
            let _ = tokens.bump();
            node
        };

        self.parse_postfix_expression(tokens, node)
    }

    pub(super) fn parse_container_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start container expression".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut elements = Vec::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            let expr = self.parse_logical_expression(tokens)?;
            self.skip_ignorable(tokens);

            if let Ok(next) = tokens.curr(false) {
                if matches!(next.key(), KEYWORD::Keyword(BUILDIN::For)) {
                    if !elements.is_empty() {
                        return Err(Box::new(ParseError::from_token(
                            &next,
                            "Rolling expressions must contain exactly one output expression"
                                .to_string(),
                        )));
                    }
                    return self.parse_rolling_expression(tokens, expr);
                }
            }

            elements.push(expr);

            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or '}' in container expression".to_string(),
            )));
        }

        if elements.len() == 1 {
            if let Some(range) = elements.pop() {
                if matches!(range, AstNode::Range { .. }) {
                    return Ok(range);
                }
                return Ok(AstNode::ContainerLiteral {
                    container_type: ContainerType::Array,
                    elements: vec![range],
                });
            }
        }

        Ok(AstNode::ContainerLiteral {
            container_type: ContainerType::Array,
            elements,
        })
    }

    pub(super) fn parse_anonymous_fun_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let routine_token = tokens.curr(false)?;
        if !matches!(routine_token.key(), KEYWORD::Keyword(BUILDIN::Fun)) {
            return Err(Box::new(ParseError::from_token(
                &routine_token,
                "Expected anonymous function expression".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.parse_anonymous_routine_after_keyword(tokens, true)
    }

    pub(super) fn parse_anonymous_pro_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let routine_token = tokens.curr(false)?;
        if !matches!(routine_token.key(), KEYWORD::Keyword(BUILDIN::Pro)) {
            return Err(Box::new(ParseError::from_token(
                &routine_token,
                "Expected anonymous procedure expression".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.parse_anonymous_routine_after_keyword(tokens, false)
    }

    pub(super) fn parse_anonymous_log_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let routine_token = tokens.curr(false)?;
        if !matches!(routine_token.key(), KEYWORD::Keyword(BUILDIN::Log)) {
            return Err(Box::new(ParseError::from_token(
                &routine_token,
                "Expected anonymous logical expression".to_string(),
            )));
        }

        let _ = tokens.bump();
        let node = self.parse_anonymous_routine_after_keyword(tokens, true)?;
        match node {
            AstNode::AnonymousFun {
                options,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            } => Ok(AstNode::AnonymousLog {
                options,
                captures,
                params,
                return_type: Some(return_type.unwrap_or(FolType::Bool)),
                error_type,
                body,
                inquiries,
            }),
            other => Ok(other),
        }
    }

    fn parse_anonymous_routine_after_keyword(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        is_function: bool,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let options = self.parse_routine_options(tokens)?;
        self.skip_ignorable(tokens);

        let open_params = tokens.curr(false)?;
        if !matches!(open_params.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_params,
                "Expected '(' after anonymous function".to_string(),
            )));
        }
        let _ = tokens.bump();

        let (params, first_untyped) = self.parse_routine_header_list(tokens)?;
        if let Some(token) = first_untyped {
            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected ':' after function parameter name".to_string(),
            )));
        }
        self.ensure_unique_parameter_names(&params, "parameter")?;

        let captures = self.parse_optional_routine_capture_list(tokens)?;
        self.ensure_unique_capture_names(&captures)?;

        self.skip_ignorable(tokens);
        let mut return_type = None;
        let mut error_type = None;
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                return_type = Some(self.parse_type_reference_tokens(tokens)?);

                self.skip_ignorable(tokens);
                if let Ok(err_sep) = tokens.curr(false) {
                    if matches!(err_sep.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        error_type = Some(self.parse_type_reference_tokens(tokens)?);
                    }
                }
            }
        }

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(
            assign.key(),
            KEYWORD::Symbol(SYMBOL::Equal) | KEYWORD::Operator(OPERATOR::Flow)
        ) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' or '=>' before anonymous function body".to_string(),
            )));
        }
        if matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            let _ = tokens.bump();
        }

        let (body, inquiries) = self.parse_named_routine_body(
            tokens,
            "Expected '{' or '=>' to start anonymous routine body",
            if is_function {
                "Expected '}' to close anonymous function body"
            } else {
                "Expected '}' to close anonymous procedure body"
            },
        )?;

        if is_function {
            Ok(AstNode::AnonymousFun {
                options,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            })
        } else {
            Ok(AstNode::AnonymousPro {
                options,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            })
        }
    }

    pub(super) fn parse_shorthand_anonymous_fun_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let open_params = tokens.curr(false)?;
        if !matches!(open_params.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_params,
                "Expected '(' to start shorthand anonymous function".to_string(),
            )));
        }
        let _ = tokens.bump();

        let (params, first_untyped) = self.parse_routine_header_list(tokens)?;
        if let Some(token) = first_untyped {
            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected ':' after function parameter name".to_string(),
            )));
        }
        self.ensure_unique_parameter_names(&params, "parameter")?;

        let captures = self.parse_optional_routine_capture_list(tokens)?;
        self.ensure_unique_capture_names(&captures)?;

        self.skip_ignorable(tokens);
        let mut return_type = None;
        let mut error_type = None;
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                return_type = Some(self.parse_type_reference_tokens(tokens)?);

                self.skip_ignorable(tokens);
                if let Ok(token) = tokens.curr(false) {
                    if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        error_type = Some(self.parse_type_reference_tokens(tokens)?);
                    }
                }
            }
        }

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start shorthand anonymous function body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let (body, inquiries) = self.parse_routine_body_with_inquiries(
            tokens,
            "Expected '}' to close shorthand anonymous function body",
        )?;

        Ok(AstNode::AnonymousFun {
            options: Vec::new(),
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
        })
    }
}
