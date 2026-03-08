use super::*;

impl AstParser {
    pub(super) fn lookahead_is_std_decl(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        if !matches!(
            tokens.curr(false).map(|token| token.key().clone()),
            Ok(KEYWORD::Keyword(BUILDIN::Std))
        ) {
            return false;
        }

        let mut significant = tokens.next_vec().into_iter().filter_map(Result::ok).filter(|token| {
            let key = token.key();
            !key.is_void() && !key.is_comment()
        });

        let Some(mut name_token) = significant.next() else {
            return false;
        };
        if matches!(name_token.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            let mut depth = 1usize;
            while depth > 0 {
                let Some(token) = significant.next() else {
                    return false;
                };
                match token.key() {
                    KEYWORD::Symbol(SYMBOL::SquarO) => depth += 1,
                    KEYWORD::Symbol(SYMBOL::SquarC) => depth -= 1,
                    _ => {}
                }
            }
            let Some(next_name) = significant.next() else {
                return false;
            };
            name_token = next_name;
        }
        if Self::token_to_named_label(&name_token).is_none() {
            return false;
        }

        matches!(
            significant.next().map(|token| token.key().clone()),
            Some(KEYWORD::Symbol(SYMBOL::Colon))
        )
    }

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
        self.parse_empty_std_options(tokens)?;
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
        if matches!(kind, StandardKind::Protocol) {
            self.parse_empty_standard_kind_options(tokens, "protocol")?;
        }
        if matches!(kind, StandardKind::Blueprint) {
            self.parse_empty_standard_kind_options(tokens, "blueprint")?;
        }
        if matches!(kind, StandardKind::Extended) {
            self.parse_empty_standard_kind_options(tokens, "extended")?;
        }

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
        let mut seen_members = HashSet::new();
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
                let member = self.parse_standard_routine_signature(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        format!("Duplicate standard member '{}'", key),
                    )));
                }
                body.push(member);
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
        let mut seen_members = HashSet::new();
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
                let members = self.parse_var_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(Box::new(ParseError::from_token(
                            &token,
                            format!("Duplicate standard member '{}'", key),
                        )));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Lab)) {
                let members = self.parse_lab_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(Box::new(ParseError::from_token(
                            &token,
                            format!("Duplicate standard member '{}'", key),
                        )));
                    }
                    body.push(member);
                }
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
        let mut seen_members = HashSet::new();
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
                let member = self.parse_standard_routine_signature(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        format!("Duplicate standard member '{}'", key),
                    )));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Var)) {
                let members = self.parse_var_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(Box::new(ParseError::from_token(
                            &token,
                            format!("Duplicate standard member '{}'", key),
                        )));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Lab)) {
                let members = self.parse_lab_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(Box::new(ParseError::from_token(
                            &token,
                            format!("Duplicate standard member '{}'", key),
                        )));
                    }
                    body.push(member);
                }
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

    fn standard_member_key(&self, node: &AstNode) -> String {
        match node {
            AstNode::FunDecl { name, params, .. } | AstNode::ProDecl { name, params, .. } => {
                format!("{}#{}", name, params.len())
            }
            AstNode::VarDecl { name, .. } => name.clone(),
            _ => String::new(),
        }
    }

    fn parse_empty_std_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<(), Box<dyn Glitch>> {
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(());
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Standard options currently support only empty brackets".to_string(),
            )));
        }
        let _ = tokens.bump();
        Ok(())
    }

    fn parse_empty_standard_kind_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        kind_name: &str,
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

        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                format!(
                    "{} standard kind options currently support only empty brackets",
                    kind_name
                ),
            )));
        }
        let _ = tokens.bump();
        Ok(())
    }
}
