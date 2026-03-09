use super::*;

impl AstParser {
    fn parse_pipe_lambda_params(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<Parameter>, Box<dyn Glitch>> {
        let mut params = Vec::new();

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if matches!(close.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
            return Ok(params);
        }

        loop {
            let mut names = Vec::new();
            loop {
                let name_token = tokens.curr(false)?;
                let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
                    Box::new(ParseError::from_token(
                        &name_token,
                        "Expected lambda parameter name".to_string(),
                    )) as Box<dyn Glitch>
                })?;
                names.push(name);
                let _ = tokens.bump();

                self.skip_ignorable(tokens);
                let token = tokens.curr(false)?;
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                    let Some(next_key) = self.next_significant_key_from_window(tokens) else {
                        return Err(Box::new(ParseError::from_token(
                            &token,
                            "Expected lambda parameter name after ','".to_string(),
                        )));
                    };

                    if matches!(next_key, KEYWORD::Symbol(SYMBOL::Pipe)) {
                        return Err(Box::new(ParseError::from_token(
                            &token,
                            "Expected lambda parameter name after ','".to_string(),
                        )));
                    }

                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    continue;
                }
                break;
            }

            self.skip_ignorable(tokens);
            let mut param_type = FolType::Any;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    param_type = self.parse_type_reference_tokens(tokens)?;
                }
            }

            self.skip_ignorable(tokens);
            let mut default = None;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    default = Some(self.parse_logical_expression(tokens)?);
                }
            }

            for name in names {
                params.push(Parameter {
                    name,
                    param_type: param_type.clone(),
                    is_borrowable: false,
                    default: default.clone(),
                });
            }

            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                continue;
            }
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
                return Ok(params);
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected ',' or '|' after lambda parameters".to_string(),
            )));
        }
    }

    pub(super) fn parse_pipe_lambda_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '|' to start lambda expression".to_string(),
            )));
        }
        let _ = tokens.bump();

        let params = self.parse_pipe_lambda_params(tokens)?;

        self.ensure_unique_parameter_names(&params, "parameter")?;

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected closing '|' after lambda parameters".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let captures = self.parse_optional_routine_capture_list(tokens)?;
        self.ensure_unique_capture_names(&captures)?;

        self.skip_ignorable(tokens);
        let (body, inquiries) = if matches!(tokens.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            let _ = tokens.bump();
            self.parse_routine_body_with_inquiries(tokens, "Expected '}' to close lambda body")?
        } else {
            (
                vec![AstNode::Return {
                    value: Some(Box::new(self.parse_logical_expression(tokens)?)),
                }],
                Vec::new(),
            )
        };

        Ok(AstNode::AnonymousFun {
            options: vec![FunOption::Mutable],
            captures,
            params,
            return_type: None,
            error_type: None,
            body,
            inquiries,
        })
    }
}
