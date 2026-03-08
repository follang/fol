use super::*;

impl AstParser {
    pub(super) fn parse_routine_body_with_inquiries(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        missing_close_message: &str,
    ) -> Result<(Vec<AstNode>, Vec<AstNode>), Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut inquiries = Vec::new();
        let mut anchor_token = None;

        for _ in 0..1024 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Where)) {
                inquiries = self.parse_optional_inquiry_clause(tokens)?;
                self.skip_ignorable(tokens);
                let close = tokens.curr(false)?;
                if matches!(close.key(), KEYWORD::Keyword(BUILDIN::Where)) {
                    return Err(Box::new(ParseError::from_token(
                        &close,
                        "Duplicate inquiry clause".to_string(),
                    )));
                }
                if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                    return Err(Box::new(ParseError::from_token(
                        &close,
                        "Expected '}' after inquiry clause".to_string(),
                    )));
                }
                let _ = tokens.bump();
                return Ok((body, inquiries));
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok((body, inquiries));
            }

            if token.key().is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    missing_close_message.to_string(),
                )));
            }

            let key = token.key();

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

            if self.lookahead_binding_alternative(tokens).is_some() {
                body.extend(self.parse_binding_alternative_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Var)) {
                body.extend(self.parse_var_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Let)) {
                body.extend(self.parse_let_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Con)) {
                body.extend(self.parse_con_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Lab)) {
                body.extend(self.parse_lab_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Use)) {
                body.extend(self.parse_use_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Seg)) {
                body.push(self.parse_seg_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Imp)) {
                body.push(self.parse_imp_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Ali)) {
                body.push(self.parse_alias_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Typ)) {
                body.push(self.parse_type_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Def)) {
                body.push(self.parse_def_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Fun)) {
                body.push(self.parse_fun_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Log)) {
                body.push(self.parse_log_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Pro)) {
                body.push(self.parse_pro_decl(tokens)?);
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

            if matches!(
                key,
                KEYWORD::Keyword(BUILDIN::Loop)
                    | KEYWORD::Keyword(BUILDIN::For)
                    | KEYWORD::Keyword(BUILDIN::Each)
            ) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                body.push(self.parse_loop_stmt(tokens)?);
                self.bump_if_no_progress(tokens, before);
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::CurlyO)) {
                body.push(self.parse_block_stmt(tokens)?);
                continue;
            }

            if (AstParser::token_can_be_logical_name(&key)
                || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
                && self.lookahead_is_assignment(tokens)
                && self.can_start_assignment(tokens)
            {
                body.push(self.parse_assignment_stmt(tokens)?);
                continue;
            }

            if (AstParser::token_can_be_logical_name(&key)
                || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
                && (self.lookahead_is_call(tokens) || self.lookahead_is_method_call(tokens))
                && self.can_start_assignment(tokens)
            {
                body.push(self.parse_call_stmt(tokens)?);
                continue;
            }

            if AstParser::token_can_be_logical_name(&key) {
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

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            missing_close_message.to_string(),
        )))
    }

    pub(super) fn parse_optional_inquiry_clause(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let where_token = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(Vec::new()),
        };

        if !matches!(where_token.key(), KEYWORD::Keyword(BUILDIN::Where)) {
            return Ok(Vec::new());
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '(' after 'where'".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let target = tokens.curr(false)?;
        if !matches!(target.key(), KEYWORD::Keyword(BUILDIN::Selfi)) {
            return Err(Box::new(ParseError::from_token(
                &target,
                "Expected 'self' in inquiry clause".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected ')' after inquiry target".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;
        if !matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open_body,
                "Expected '{' to start inquiry body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.parse_inquiry_body(tokens)
    }

    fn parse_inquiry_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut anchor_token = None;

        for _ in 0..512 {
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
                    "Expected '}' to close inquiry body".to_string(),
                )));
            }

            body.push(self.parse_logical_expression(tokens)?);
            self.consume_optional_semicolon(tokens);
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            "Inquiry body exceeded parser limit".to_string(),
        )))
    }
}
