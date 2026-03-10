use super::*;

impl AstParser {
    fn lookahead_is_record_init_field(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let current = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return false,
        };
        if Self::token_to_named_label(&current).is_none() {
            return false;
        }

        matches!(
            self.next_significant_key_from_window(tokens),
            Some(KEYWORD::Symbol(SYMBOL::Equal))
        )
    }

    fn parse_record_init_fields_after_open(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut fields = Vec::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            let name = Self::token_to_named_label(&token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &token,
                    "Expected field name in record initializer".to_string(),
                )) as Box<dyn Glitch>
            })?;
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let equal = tokens.curr(false)?;
            if !matches!(equal.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                return Err(Box::new(ParseError::from_token(
                    &equal,
                    "Expected '=' after record initializer field name".to_string(),
                )));
            }
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let value = self.parse_logical_expression(tokens)?;
            fields.push(crate::ast::RecordInitField { name, value });
            self.skip_ignorable(tokens);

            let sep = tokens.curr(false)?;
            if matches!(
                sep.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                if matches!(
                    tokens.curr(false).map(|token| token.key()),
                    Ok(KEYWORD::Symbol(SYMBOL::CurlyC))
                ) {
                    let _ = tokens.bump();
                    break;
                }
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or '}' in record initializer".to_string(),
            )));
        }

        Ok(AstNode::RecordInit { fields })
    }

    fn lookahead_is_spawn_expression(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let current = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return false,
        };
        if !matches!(current.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return false;
        }

        let mut found = Vec::new();
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };
            if token.key().is_void() || token.key().is_comment() {
                continue;
            }
            found.push(token.key());
            if found.len() == 2 {
                break;
            }
        }

        matches!(
            found.as_slice(),
            [KEYWORD::Symbol(SYMBOL::AngleC), KEYWORD::Symbol(SYMBOL::SquarC)]
        )
    }

    fn lookahead_is_match_expression(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let current = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return false,
        };
        if !matches!(
            current.key(),
            KEYWORD::Keyword(BUILDIN::If) | KEYWORD::Keyword(BUILDIN::When)
        ) {
            return false;
        }

        let mut round_depth = 0usize;
        let mut saw_open_round = false;
        let mut saw_close_round = false;

        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if !saw_open_round {
                if matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                    saw_open_round = true;
                    round_depth = 1;
                    continue;
                }
                return false;
            }

            if !saw_close_round {
                if matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                    round_depth += 1;
                    continue;
                }
                if matches!(key, KEYWORD::Symbol(SYMBOL::RoundC)) {
                    round_depth -= 1;
                    if round_depth == 0 {
                        saw_close_round = true;
                    }
                    continue;
                }
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::CurlyO));
        }

        false
    }

    fn parse_match_arrow_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let flow = tokens.curr(false)?;
        if !matches!(
            flow.key(),
            KEYWORD::Operator(OPERATOR::Flow2) | KEYWORD::Operator(OPERATOR::Flow)
        ) {
            return Err(Box::new(ParseError::from_token(
                &flow,
                "Expected '->' or '=>' in matching expression".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let expr = self.parse_logical_expression(tokens)?;
        self.skip_ignorable(tokens);
        if matches!(
            tokens.curr(false).map(|token| token.key()),
            Ok(KEYWORD::Symbol(SYMBOL::Semi))
        ) {
            let _ = tokens.bump();
        }

        Ok(vec![expr])
    }

    fn parse_match_expression_has_members(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut members = Vec::new();
        for _ in 0..64 {
            members.push(self.parse_logical_expression(tokens)?);
            self.skip_ignorable(tokens);

            let next = tokens.curr(false)?;
            if matches!(
                next.key(),
                KEYWORD::Operator(OPERATOR::Flow2) | KEYWORD::Operator(OPERATOR::Flow)
            ) {
                break;
            }
            if matches!(
                next.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &next,
                "Expected ',', ';', or '->' in matching has-case".to_string(),
            )));
        }

        if members.len() == 1 {
            Ok(members.into_iter().next().expect("single has member"))
        } else {
            Ok(AstNode::ContainerLiteral {
                container_type: ContainerType::Set,
                elements: members,
            })
        }
    }

    fn parse_match_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let keyword = tokens.curr(false)?;
        if !matches!(
            keyword.key(),
            KEYWORD::Keyword(BUILDIN::If) | KEYWORD::Keyword(BUILDIN::When)
        ) {
            return Err(Box::new(ParseError::from_token(
                &keyword,
                "Expected matching expression".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '(' after matching keyword".to_string(),
            )));
        }
        let _ = tokens.bump();

        let expr = self.parse_logical_expression(tokens)?;
        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected ')' after matching expression condition".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let open_cases = tokens.curr(false)?;
        if !matches!(open_cases.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_cases,
                "Expected '{' to start matching expression cases".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut cases = Vec::new();
        let mut default = None;

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::In)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                let range = self.parse_logical_expression(tokens)?;
                let body = self.parse_match_arrow_body(tokens)?;
                cases.push(WhenCase::In { range, body });
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Is)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                let value = self.parse_logical_expression(tokens)?;
                let body = self.parse_match_arrow_body(tokens)?;
                cases.push(WhenCase::Is { value, body });
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Has)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                let member = self.parse_match_expression_has_members(tokens)?;
                let body = self.parse_match_arrow_body(tokens)?;
                cases.push(WhenCase::Has { member, body });
                continue;
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Star)) {
                let _ = tokens.bump();
                default = Some(self.parse_match_arrow_body(tokens)?);
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected in/is/has/* case in matching expression".to_string(),
            )));
        }

        Ok(AstNode::When {
            expr: Box::new(expr),
            cases,
            default,
        })
    }

    fn lookahead_is_shorthand_anonymous_fun(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let mut params_depth = 1usize;
        let mut params_closed = false;
        let mut capture_depth = 0usize;
        let mut in_type_clause = false;
        let mut type_round_depth = 0usize;
        let mut type_square_depth = 0usize;
        let mut type_curly_depth = 0usize;

        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if !params_closed {
                match key {
                    KEYWORD::Symbol(SYMBOL::RoundO) => {
                        params_depth += 1;
                        continue;
                    }
                    KEYWORD::Symbol(SYMBOL::RoundC) => {
                        if params_depth == 0 {
                            return false;
                        }
                        params_depth -= 1;
                        if params_depth == 0 {
                            params_closed = true;
                        }
                        continue;
                    }
                    _ => continue,
                }
            }

            if capture_depth > 0 {
                match key {
                    KEYWORD::Symbol(SYMBOL::SquarO) => capture_depth += 1,
                    KEYWORD::Symbol(SYMBOL::SquarC) => capture_depth -= 1,
                    _ => {}
                }
                continue;
            }

            if in_type_clause {
                match key {
                    KEYWORD::Symbol(SYMBOL::RoundO) => {
                        type_round_depth += 1;
                        continue;
                    }
                    KEYWORD::Symbol(SYMBOL::RoundC) if type_round_depth > 0 => {
                        type_round_depth -= 1;
                        continue;
                    }
                    KEYWORD::Symbol(SYMBOL::SquarO) => {
                        type_square_depth += 1;
                        continue;
                    }
                    KEYWORD::Symbol(SYMBOL::SquarC) if type_square_depth > 0 => {
                        type_square_depth -= 1;
                        continue;
                    }
                    KEYWORD::Symbol(SYMBOL::CurlyO) => {
                        if type_round_depth == 0
                            && type_square_depth == 0
                            && type_curly_depth == 0
                            && !matches!(
                                self.next_significant_key_from_window(tokens),
                                Some(KEYWORD::Keyword(BUILDIN::Fun))
                            )
                        {
                            return true;
                        }
                        type_curly_depth += 1;
                        continue;
                    }
                    KEYWORD::Symbol(SYMBOL::CurlyC) if type_curly_depth > 0 => {
                        type_curly_depth -= 1;
                        continue;
                    }
                    KEYWORD::Operator(OPERATOR::Flow)
                        if type_round_depth == 0
                            && type_square_depth == 0
                            && type_curly_depth == 0 =>
                    {
                        return true;
                    }
                    KEYWORD::Symbol(SYMBOL::Colon)
                        if type_round_depth == 0
                            && type_square_depth == 0
                            && type_curly_depth == 0 =>
                    {
                        continue;
                    }
                    _ => continue,
                }
            }

            match key {
                KEYWORD::Symbol(SYMBOL::SquarO) => {
                    capture_depth = 1;
                }
                KEYWORD::Symbol(SYMBOL::Colon) => {
                    in_type_clause = true;
                }
                KEYWORD::Symbol(SYMBOL::CurlyO) | KEYWORD::Operator(OPERATOR::Flow) => {
                    return true;
                }
                _ => {
                    return false;
                }
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

        if self.lookahead_is_spawn_expression(tokens) {
            self.consume_significant_token(tokens);

            let angle = tokens.curr(false)?;
            if !matches!(angle.key(), KEYWORD::Symbol(SYMBOL::AngleC)) {
                return Err(Box::new(ParseError::from_token(
                    &angle,
                    "Expected '>' in spawn marker".to_string(),
                )));
            }
            self.consume_significant_token(tokens);

            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected closing ']' in spawn marker".to_string(),
                )));
            }
            self.consume_significant_token(tokens);
            self.skip_ignorable(tokens);

            let task = self.parse_primary_expression(tokens)?;
            return Ok(AstNode::Spawn {
                task: Box::new(task),
            });
        }

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

        let node = if matches!(
            token.key(),
            KEYWORD::Keyword(BUILDIN::If) | KEYWORD::Keyword(BUILDIN::When)
        ) && self.lookahead_is_match_expression(tokens)
        {
            self.parse_match_expression(tokens)?
        } else if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Dot))
            && self.lookahead_is_dot_builtin_call(tokens)
        {
            self.parse_dot_builtin_call_expr(tokens)?
        } else if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
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

        self.skip_ignorable(tokens);
        if matches!(
            tokens.curr(false).map(|token| token.key()),
            Ok(KEYWORD::Symbol(SYMBOL::CurlyC))
        ) {
            let _ = tokens.bump();
            return Ok(AstNode::ContainerLiteral {
                container_type: ContainerType::Array,
                elements: Vec::new(),
            });
        }

        if self.lookahead_is_record_init_field(tokens) {
            return self.parse_record_init_fields_after_open(tokens);
        }

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
            if matches!(
                sep.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                if matches!(
                    tokens.curr(false).map(|token| token.key()),
                    Ok(KEYWORD::Symbol(SYMBOL::CurlyC))
                ) {
                    let _ = tokens.bump();
                    break;
                }
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or '}' in container expression".to_string(),
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

        let (body, inquiries) = self.parse_named_routine_body(
            tokens,
            "Expected '{' or '=>' to start shorthand anonymous function body",
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
