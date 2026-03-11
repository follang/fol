use super::*;

impl AstParser {
    pub(super) fn parse_fun_decl(
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
        let options = self.parse_routine_options(tokens)?;
        self.skip_ignorable(tokens);

        let (_receiver_type, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected function name after 'fun'",
        )?;

        self.skip_ignorable(tokens);
        let alt_generics = if self.lookahead_parenthesized_generic_header_before_colon(tokens) {
            let open = tokens.curr(false)?;
            if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                Vec::new()
            } else {
                let _ = tokens.bump();
                self.parse_generic_list(tokens)?
            }
        } else {
            Vec::new()
        };

        self.skip_ignorable(tokens);
        if matches!(
            tokens.curr(false).map(|token| token.key().clone()),
            Ok(KEYWORD::Symbol(SYMBOL::Colon))
        ) {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            let return_type = Some(self.parse_type_reference_tokens(tokens)?);
            let mut error_type = None;

            self.skip_ignorable(tokens);
            if let Ok(err_sep) = tokens.curr(false) {
                if matches!(err_sep.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    error_type = Some(self.parse_type_reference_tokens(tokens)?);
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
                    "Expected '=' or '=>' before function body".to_string(),
                )));
            }
            if matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                let _ = tokens.bump();
            }

            self.skip_ignorable(tokens);
            let mut params = Vec::new();
            if matches!(tokens.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                let _ = tokens.bump();
                let (parsed_params, first_untyped) = self.parse_routine_header_list(tokens)?;
                if let Some(token) = first_untyped {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected ':' after function parameter name".to_string(),
                    )));
                }
                self.ensure_unique_parameter_names(&parsed_params, "parameter")?;
                params = parsed_params;
            }

            self.skip_ignorable(tokens);
            let captures = self.parse_optional_routine_capture_list(tokens)?;
            self.ensure_unique_capture_names(&captures)?;

            let (body, inquiries) = self.parse_named_routine_body(
                tokens,
                "Expected '{' or '=>' to start function body",
                "Expected '}' to close function body",
            )?;

            return Ok(AstNode::FunDecl {
                options,
                generics: alt_generics,
                name,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            });
        }

        let (generics, params) =
            self.parse_routine_generics_and_params(tokens, "Expected '(' after function name")?;
        self.skip_ignorable(tokens);
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
                "Expected '=' or '=>' before function body".to_string(),
            )));
        }
        if matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            let _ = tokens.bump();
        }

        let (body, inquiries) = self.parse_named_routine_body(
            tokens,
            "Expected '{' or '=>' to start function body",
            "Expected '}' to close function body",
        )?;

        Ok(AstNode::FunDecl {
            options,
            generics,
            name,
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
        })
    }

    pub(super) fn parse_log_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let log_token = tokens.curr(false)?;
        if !matches!(log_token.key(), KEYWORD::Keyword(BUILDIN::Log)) {
            return Err(Box::new(ParseError::from_token(
                &log_token,
                "Expected 'log' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let options = self.parse_routine_options(tokens)?;
        self.skip_ignorable(tokens);

        let (_receiver_type, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected logical name after 'log'",
        )?;

        self.skip_ignorable(tokens);
        let alt_generics = if self.lookahead_parenthesized_generic_header_before_colon(tokens) {
            let open = tokens.curr(false)?;
            if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                Vec::new()
            } else {
                let _ = tokens.bump();
                self.parse_generic_list(tokens)?
            }
        } else {
            Vec::new()
        };

        self.skip_ignorable(tokens);
        if matches!(
            tokens.curr(false).map(|token| token.key().clone()),
            Ok(KEYWORD::Symbol(SYMBOL::Colon))
        ) {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            let return_type = Some(self.parse_type_reference_tokens(tokens)?);
            let mut error_type = None;

            self.skip_ignorable(tokens);
            if let Ok(err_sep) = tokens.curr(false) {
                if matches!(err_sep.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    error_type = Some(self.parse_type_reference_tokens(tokens)?);
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
                    "Expected '=' or '=>' before logical body".to_string(),
                )));
            }
            if matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                let _ = tokens.bump();
            }

            self.skip_ignorable(tokens);
            let mut params = Vec::new();
            if matches!(tokens.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                let _ = tokens.bump();
                let (parsed_params, first_untyped) = self.parse_routine_header_list(tokens)?;
                if let Some(token) = first_untyped {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected ':' after function parameter name".to_string(),
                    )));
                }
                self.ensure_unique_parameter_names(&parsed_params, "parameter")?;
                params = parsed_params;
            }

            self.skip_ignorable(tokens);
            let captures = self.parse_optional_routine_capture_list(tokens)?;
            self.ensure_unique_capture_names(&captures)?;

            let (body, inquiries) = self.parse_named_routine_body(
                tokens,
                "Expected '{' or '=>' to start logical body",
                "Expected '}' to close logical body",
            )?;

            return Ok(AstNode::FunDecl {
                options,
                generics: alt_generics,
                name,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            });
        }

        let (generics, params) =
            self.parse_routine_generics_and_params(tokens, "Expected '(' after logical name")?;
        self.skip_ignorable(tokens);
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
                "Expected '=' or '=>' before logical body".to_string(),
            )));
        }
        if matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            let _ = tokens.bump();
        }

        let (body, inquiries) = self.parse_named_routine_body(
            tokens,
            "Expected '{' or '=>' to start logical body",
            "Expected '}' to close logical body",
        )?;

        Ok(AstNode::FunDecl {
            options,
            generics,
            name,
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
        })
    }

    pub(super) fn parse_pro_decl(
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
        let options = self.parse_routine_options(tokens)?;
        self.skip_ignorable(tokens);

        let (_receiver_type, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected procedure name after 'pro'",
        )?;

        self.skip_ignorable(tokens);
        let alt_generics = if self.lookahead_parenthesized_generic_header_before_colon(tokens) {
            let open = tokens.curr(false)?;
            if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                Vec::new()
            } else {
                let _ = tokens.bump();
                self.parse_generic_list(tokens)?
            }
        } else {
            Vec::new()
        };

        self.skip_ignorable(tokens);
        if matches!(
            tokens.curr(false).map(|token| token.key().clone()),
            Ok(KEYWORD::Symbol(SYMBOL::Colon))
        ) {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            let return_type = Some(self.parse_type_reference_tokens(tokens)?);
            let mut error_type = None;

            self.skip_ignorable(tokens);
            if let Ok(err_sep) = tokens.curr(false) {
                if matches!(err_sep.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    error_type = Some(self.parse_type_reference_tokens(tokens)?);
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
                    "Expected '=' or '=>' before procedure body".to_string(),
                )));
            }
            if matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                let _ = tokens.bump();
            }

            self.skip_ignorable(tokens);
            let mut params = Vec::new();
            if matches!(tokens.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                let _ = tokens.bump();
                let (parsed_params, first_untyped) = self.parse_routine_header_list(tokens)?;
                if let Some(token) = first_untyped {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected ':' after function parameter name".to_string(),
                    )));
                }
                self.ensure_unique_parameter_names(&parsed_params, "parameter")?;
                params = parsed_params;
            }

            self.skip_ignorable(tokens);
            let captures = self.parse_optional_routine_capture_list(tokens)?;
            self.ensure_unique_capture_names(&captures)?;

            let (body, inquiries) = self.parse_named_routine_body(
                tokens,
                "Expected '{' or '=>' to start procedure body",
                "Expected '}' to close procedure body",
            )?;

            return Ok(AstNode::ProDecl {
                options,
                generics: alt_generics,
                name,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            });
        }

        let (generics, params) =
            self.parse_routine_generics_and_params(tokens, "Expected '(' after procedure name")?;
        self.skip_ignorable(tokens);
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
                "Expected '=' or '=>' before procedure body".to_string(),
            )));
        }
        if matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            let _ = tokens.bump();
        }

        let (body, inquiries) = self.parse_named_routine_body(
            tokens,
            "Expected '{' or '=>' to start procedure body",
            "Expected '}' to close procedure body",
        )?;

        Ok(AstNode::ProDecl {
            options,
            generics,
            name,
            captures,
            params,
            return_type,
            error_type,
            body,
            inquiries,
        })
    }

    pub(super) fn parse_routine_generics_and_params(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        missing_open_message: &str,
    ) -> Result<(Vec<Generic>, Vec<Parameter>), Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open_paren = tokens.curr(false)?;
        if !matches!(open_paren.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_paren,
                missing_open_message.to_string(),
            )));
        }
        let _ = tokens.bump();

        let (first_list, first_untyped) = self.parse_routine_header_list(tokens)?;
        self.skip_ignorable(tokens);

        let next = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => {
                if let Some(token) = first_untyped {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected ':' after parameter name".to_string(),
                    )));
                }
                self.ensure_unique_parameter_names(&first_list, "parameter")?;
                return Ok((Vec::new(), first_list));
            }
        };

        if !matches!(next.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            if let Some(token) = first_untyped {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Expected ':' after parameter name".to_string(),
                )));
            }
            self.ensure_unique_parameter_names(&first_list, "parameter")?;
            return Ok((Vec::new(), first_list));
        }

        self.ensure_unique_parameter_names(&first_list, "generic")?;
        let generics = self.parameters_to_generics(first_list)?;
        let _ = tokens.bump();
        let params = self.parse_parameter_list(tokens)?;
        Ok((generics, params))
    }
}
