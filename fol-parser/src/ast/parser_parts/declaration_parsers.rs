use super::*;

impl AstParser {
    pub(super) fn parse_def_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let def_token = tokens.curr(false)?;
        if !matches!(def_token.key(), KEYWORD::Keyword(BUILDIN::Def)) {
            return Err(Box::new(ParseError::from_token(
                &def_token,
                "Expected 'def' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let options = self.parse_decl_visibility_options(tokens, "definition")?;
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = match name_token.key() {
            key if key.is_ident() || key.is_buildin() => name_token.con().trim().to_string(),
            KEYWORD::Literal(LITERAL::Stringy) => name_token
                .con()
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string(),
            _ => {
                return Err(Box::new(ParseError::from_token(
                    &name_token,
                    "Expected definition name".to_string(),
                )));
            }
        };
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' after definition name".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let def_type_token = tokens.curr(false)?;
        let def_type = self.parse_type_reference_tokens(tokens)?;
        if !matches!(
            def_type,
            FolType::Module { .. } | FolType::Block { .. } | FolType::Test { .. }
        ) {
            return Err(Box::new(ParseError::from_token(
                &def_type_token,
                format!(
                    "Definition declarations currently support only mod[...], blk[...], or tst[...] types, found '{}'",
                    Self::fol_type_label(&def_type)
                ),
            )));
        }

        self.skip_ignorable(tokens);
        let next = tokens.curr(false)?;
        if !matches!(next.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            if matches!(def_type, FolType::Block { .. }) {
                self.consume_optional_semicolon(tokens);
                return Ok(AstNode::DefDecl {
                    options,
                    name,
                    def_type,
                    body: Vec::new(),
                });
            }

            return Err(Box::new(ParseError::from_token(
                &next,
                "Expected '=' before definition body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start definition body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let body = self.parse_block_body(tokens, "Expected '}' to close definition body")?;
        self.consume_optional_semicolon(tokens);

        Ok(AstNode::DefDecl {
            options,
            name,
            def_type,
            body,
        })
    }

    pub(super) fn parse_alias_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let ali_token = tokens.curr(false)?;
        if !matches!(ali_token.key(), KEYWORD::Keyword(BUILDIN::Ali)) {
            return Err(Box::new(ParseError::from_token(
                &ali_token,
                "Expected 'ali' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &name_token,
                "Expected alias declaration name".to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' after alias name".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let target = self.parse_type_reference_tokens(tokens)?;

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::AliasDecl { name, target })
    }

    pub(super) fn parse_type_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let typ_token = tokens.curr(false)?;
        if !matches!(typ_token.key(), KEYWORD::Keyword(BUILDIN::Typ)) {
            return Err(Box::new(ParseError::from_token(
                &typ_token,
                "Expected 'typ' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let options = self.parse_type_options(tokens)?;
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &name_token,
                "Expected type declaration name".to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let generics = self.parse_type_generic_header(tokens)?;
        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' after type name".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let type_def = if tokens.curr(false)?.con().trim() == "ent" {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            self.parse_empty_type_marker_brackets(tokens, "entry")?;
            self.skip_ignorable(tokens);

            let assign = tokens.curr(false)?;
            if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                return Err(Box::new(ParseError::from_token(
                    &assign,
                    "Expected '=' after entry type marker".to_string(),
                )));
            }
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            self.parse_entry_type_definition(tokens)?
        } else if tokens.curr(false)?.con().trim() == "rec" {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            self.parse_empty_type_marker_brackets(tokens, "record")?;
            self.skip_ignorable(tokens);

            let assign = tokens.curr(false)?;
            if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                return Err(Box::new(ParseError::from_token(
                    &assign,
                    "Expected '=' after record type marker".to_string(),
                )));
            }
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            self.parse_record_type_definition(tokens)?
        } else if matches!(tokens.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyO))
            && !matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Keyword(BUILDIN::Fun))
            )
        {
            self.parse_record_type_definition(tokens)?
        } else {
            let target = self.parse_type_reference_tokens(tokens)?;
            TypeDefinition::Alias { target }
        };

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::TypeDecl {
            options,
            generics,
            name,
            type_def,
        })
    }

    pub(super) fn parse_empty_type_marker_brackets(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        marker_name: &str,
    ) -> Result<(), Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(());
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let token = tokens.curr(false)?;
        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            let _ = tokens.bump();
            return Ok(());
        }

        Err(Box::new(ParseError::from_token(
            &token,
            format!("Unknown {} type marker option", marker_name),
        )))
    }

    pub(super) fn parse_type_generic_header(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<Generic>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(Vec::new()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Ok(Vec::new());
        }
        let _ = tokens.bump();

        self.parse_generic_list(tokens)
    }

    pub(super) fn parse_type_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<TypeOption>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(Vec::new()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(Vec::new());
        }
        let _ = tokens.bump();

        let mut options = Vec::new();
        for _ in 0..16 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(options);
            }

            let option = match token.con().trim() {
                "+" | "pub" | "exp" | "export" => TypeOption::Export,
                "set" => TypeOption::Set,
                "get" => TypeOption::Get,
                "nothing" | "non" => TypeOption::Nothing,
                "ext" => TypeOption::Extension,
                _ => {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Unknown type option".to_string(),
                    )))
                }
            };
            options.push(option);
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(options);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or ']' in type options".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Type options exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }

    pub(super) fn parse_use_path(
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

            let segment = match token.key() {
                KEYWORD::Literal(LITERAL::Stringy) => token
                    .con()
                    .trim()
                    .trim_matches(|c| c == '"' || c == '\''),
                _ => token.con().trim(),
            };
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

        let (receiver_type, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected function name after 'fun'",
        )?;

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

            if let Some(rt) = return_type.as_ref() {
                self.register_routine_return_type(
                    &name,
                    0,
                    receiver_type.as_ref(),
                    rt,
                    &fun_token,
                )?;
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

            self.skip_ignorable(tokens);
            let open_body = tokens.curr(false)?;
            if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                return Err(Box::new(ParseError::from_token(
                    &open_body,
                    "Expected '{' to start function body".to_string(),
                )));
            }
            let _ = tokens.bump();

            let (body, inquiries) = self.parse_routine_body_with_inquiries(
                tokens,
                "Expected '}' to close function body",
            )?;
            let parameter_types = Self::parameter_type_map(&params);
            let routine_returns = self.routine_return_types.borrow().clone();
            Self::validate_report_usage(
                &body,
                error_type.as_ref(),
                &parameter_types,
                &routine_returns,
                &fun_token,
            )?;

            return Ok(AstNode::FunDecl {
                options,
                generics: Vec::new(),
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

        if let Some(rt) = return_type.as_ref() {
            self.register_routine_return_type(
                &name,
                params.len(),
                receiver_type.as_ref(),
                rt,
                &fun_token,
            )?;
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

        let (body, inquiries) =
            self.parse_routine_body_with_inquiries(tokens, "Expected '}' to close function body")?;
        let parameter_types = Self::parameter_type_map(&params);
        let routine_returns = self.routine_return_types.borrow().clone();
        Self::validate_report_usage(
            &body,
            error_type.as_ref(),
            &parameter_types,
            &routine_returns,
            &fun_token,
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

        let (receiver_type, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected logical name after 'log'",
        )?;

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

            if let Some(rt) = return_type.as_ref() {
                self.register_routine_return_type(
                    &name,
                    0,
                    receiver_type.as_ref(),
                    rt,
                    &log_token,
                )?;
            }

            self.skip_ignorable(tokens);
            let assign = tokens.curr(false)?;
            if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                return Err(Box::new(ParseError::from_token(
                    &assign,
                    "Expected '=' before logical body".to_string(),
                )));
            }
            let _ = tokens.bump();

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
            let open_body = tokens.curr(false)?;
            if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                return Err(Box::new(ParseError::from_token(
                    &open_body,
                    "Expected '{' to start logical body".to_string(),
                )));
            }
            let _ = tokens.bump();

            let (body, inquiries) = self.parse_routine_body_with_inquiries(
                tokens,
                "Expected '}' to close logical body",
            )?;
            let parameter_types = Self::parameter_type_map(&params);
            let routine_returns = self.routine_return_types.borrow().clone();
            Self::validate_report_usage(
                &body,
                error_type.as_ref(),
                &parameter_types,
                &routine_returns,
                &log_token,
            )?;

            return Ok(AstNode::FunDecl {
                options,
                generics: Vec::new(),
                name,
                captures: Vec::new(),
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

        if let Some(rt) = return_type.as_ref() {
            self.register_routine_return_type(
                &name,
                params.len(),
                receiver_type.as_ref(),
                rt,
                &log_token,
            )?;
        }

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' before logical body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start logical body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let (body, inquiries) =
            self.parse_routine_body_with_inquiries(tokens, "Expected '}' to close logical body")?;
        let parameter_types = Self::parameter_type_map(&params);
        let routine_returns = self.routine_return_types.borrow().clone();
        Self::validate_report_usage(
            &body,
            error_type.as_ref(),
            &parameter_types,
            &routine_returns,
            &log_token,
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

        let (receiver_type, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected procedure name after 'pro'",
        )?;

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

            if let Some(rt) = return_type.as_ref() {
                self.register_routine_return_type(
                    &name,
                    0,
                    receiver_type.as_ref(),
                    rt,
                    &pro_token,
                )?;
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
            let open_body = tokens.curr(false)?;
            if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                return Err(Box::new(ParseError::from_token(
                    &open_body,
                    "Expected '{' to start procedure body".to_string(),
                )));
            }
            let _ = tokens.bump();

            let (body, inquiries) = self.parse_routine_body_with_inquiries(
                tokens,
                "Expected '}' to close procedure body",
            )?;
            let parameter_types = Self::parameter_type_map(&params);
            let routine_returns = self.routine_return_types.borrow().clone();
            Self::validate_report_usage(
                &body,
                error_type.as_ref(),
                &parameter_types,
                &routine_returns,
                &pro_token,
            )?;

            return Ok(AstNode::ProDecl {
                options,
                generics: Vec::new(),
                name,
                captures: Vec::new(),
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

        if let Some(rt) = return_type.as_ref() {
            self.register_routine_return_type(
                &name,
                params.len(),
                receiver_type.as_ref(),
                rt,
                &pro_token,
            )?;
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

        let (body, inquiries) =
            self.parse_routine_body_with_inquiries(tokens, "Expected '}' to close procedure body")?;
        let parameter_types = Self::parameter_type_map(&params);
        let routine_returns = self.routine_return_types.borrow().clone();
        Self::validate_report_usage(
            &body,
            error_type.as_ref(),
            &parameter_types,
            &routine_returns,
            &pro_token,
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
