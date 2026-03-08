use super::*;

impl AstParser {
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

        let node = if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Fun)) {
            self.parse_anonymous_fun_expr(tokens)?
        } else if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return self.parse_container_expression(tokens);
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
        let fun_token = tokens.curr(false)?;
        if !matches!(fun_token.key(), KEYWORD::Keyword(BUILDIN::Fun)) {
            return Err(Box::new(ParseError::from_token(
                &fun_token,
                "Expected anonymous function expression".to_string(),
            )));
        }

        let _ = tokens.bump();
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
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' before anonymous function body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start anonymous function body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let (body, _inquiries) = self.parse_routine_body_with_inquiries(
            tokens,
            "Expected '}' to close anonymous function body",
        )?;

        Ok(AstNode::AnonymousFun {
            options,
            params,
            return_type,
            error_type,
            body,
        })
    }
}
