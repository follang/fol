use super::*;

impl AstParser {
    pub(super) fn parse_std_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let std_token = tokens.curr(false)?;
        if !matches!(std_token.key(), KEYWORD::Keyword(BUILDIN::Std)) {
            return Err(Box::new(ParseError::from_token(
                &std_token,
                "Expected 'std' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &name_token,
                "Expected standard name after 'std'".to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' after standard name".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let kind_token = tokens.curr(false)?;
        let kind = match kind_token.con().trim() {
            "pro" => StandardKind::Protocol,
            "blu" => StandardKind::Blueprint,
            "ext" => StandardKind::Extended,
            other => {
                return Err(Box::new(ParseError::from_token(
                    &kind_token,
                    format!("Unknown standard kind '{}'", other),
                )))
            }
        };
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' before standard body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start standard body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let body = match kind {
            StandardKind::Protocol => self.parse_standard_protocol_body(tokens)?,
            StandardKind::Blueprint => self.parse_standard_blueprint_body(tokens)?,
            StandardKind::Extended => self.parse_standard_extended_body(tokens)?,
        };
        self.consume_optional_semicolon(tokens);

        Ok(AstNode::StdDecl { name, kind, body })
    }

    fn parse_standard_protocol_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut anchor_token = None;

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if token.key().is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    "Expected '}' to close standard body".to_string(),
                )));
            }

            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Log)
                    | KEYWORD::Keyword(BUILDIN::Pro)
            ) {
                body.push(self.parse_standard_routine_signature(tokens)?);
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Protocol standards currently support only routine signatures".to_string(),
            )));
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            "Standard body exceeded parser limit".to_string(),
        )))
    }

    fn parse_standard_blueprint_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut anchor_token = None;

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if token.key().is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    "Expected '}' to close standard body".to_string(),
                )));
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Var)) {
                body.extend(self.parse_var_decl(tokens)?);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Lab)) {
                body.extend(self.parse_lab_decl(tokens)?);
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Blueprint standards currently support only field declarations".to_string(),
            )));
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            "Standard body exceeded parser limit".to_string(),
        )))
    }

    fn parse_standard_extended_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut anchor_token = None;

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if token.key().is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    "Expected '}' to close standard body".to_string(),
                )));
            }

            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Log)
                    | KEYWORD::Keyword(BUILDIN::Pro)
            ) {
                body.push(self.parse_standard_routine_signature(tokens)?);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Var)) {
                body.extend(self.parse_var_decl(tokens)?);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Lab)) {
                body.extend(self.parse_lab_decl(tokens)?);
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Extended standards currently support only routine signatures and field declarations"
                    .to_string(),
            )));
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            "Standard body exceeded parser limit".to_string(),
        )))
    }

    fn parse_standard_routine_signature(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let routine_token = tokens.curr(false)?;
        let routine_kind = routine_token.key().clone();
        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let options = self.parse_routine_options(tokens)?;
        self.skip_ignorable(tokens);

        let (_, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected routine name in standard signature",
        )?;
        let (generics, params) =
            self.parse_routine_generics_and_params(tokens, "Expected '(' after routine name")?;

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
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::Semi)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected ';' after standard routine signature".to_string(),
            )));
        }
        let _ = tokens.bump();

        Ok(match routine_kind {
            KEYWORD::Keyword(BUILDIN::Pro) => AstNode::ProDecl {
                options,
                generics,
                name,
                params,
                return_type,
                error_type,
                body: Vec::new(),
                inquiries: Vec::new(),
            },
            _ => AstNode::FunDecl {
                options,
                generics,
                name,
                params,
                return_type,
                error_type,
                body: Vec::new(),
                inquiries: Vec::new(),
            },
        })
    }
}
