use super::*;

impl AstParser {
    pub(super) fn parse_rolling_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        expr: AstNode,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let for_token = tokens.curr(false)?;
        if !matches!(for_token.key(), KEYWORD::Keyword(BUILDIN::For)) {
            return Err(Box::new(ParseError::from_token(
                &for_token,
                "Expected 'for' in rolling expression".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let bindings = self.parse_rolling_bindings(tokens)?;
        self.skip_ignorable(tokens);

        let condition = if let Ok(token) = tokens.curr(false) {
            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::If) | KEYWORD::Keyword(BUILDIN::When)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                Some(Box::new(self.parse_logical_expression(tokens)?))
            } else {
                None
            }
        } else {
            None
        };

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected '}' to close rolling expression".to_string(),
            )));
        }
        let _ = tokens.bump();

        Ok(AstNode::Rolling {
            expr: Box::new(expr),
            bindings,
            condition,
        })
    }

    pub(super) fn parse_rolling_bindings(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<RollingBinding>, Box<dyn Glitch>> {
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                let _ = tokens.bump();
                let mut bindings = Vec::new();
                let mut seen_names = HashSet::new();

                for _ in 0..64 {
                    self.skip_ignorable(tokens);
                    let token = tokens.curr(false)?;
                    if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                        let _ = tokens.bump();
                        return Ok(bindings);
                    }

                    let binding = self.parse_rolling_binding(tokens)?;
                    if !seen_names.insert(binding.name.clone()) {
                        return Err(Box::new(ParseError {
                            message: format!("Duplicate rolling binding '{}'", binding.name),
                            file: None,
                            line: 0,
                            column: 0,
                            length: 0,
                        }));
                    }
                    bindings.push(binding);
                    self.skip_ignorable(tokens);

                    let sep = tokens.curr(false)?;
                    if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                        let _ = tokens.bump();
                        continue;
                    }
                    if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                        let _ = tokens.bump();
                        return Ok(bindings);
                    }

                    return Err(Box::new(ParseError::from_token(
                        &sep,
                        "Expected ',' or ')' in rolling bindings".to_string(),
                    )));
                }

                return Err(Box::new(ParseError {
                    message: "Rolling binding list exceeded parser limit".to_string(),
                    file: None,
                    line: 0,
                    column: 0,
                    length: 0,
                }));
            }
        }

        let mut bindings = Vec::new();
        let mut seen_names = HashSet::new();
        for _ in 0..64 {
            let binding = self.parse_rolling_binding(tokens)?;
            if !seen_names.insert(binding.name.clone()) {
                return Err(Box::new(ParseError {
                    message: format!("Duplicate rolling binding '{}'", binding.name),
                    file: None,
                    line: 0,
                    column: 0,
                    length: 0,
                }));
            }
            bindings.push(binding);
            self.skip_ignorable(tokens);

            let Ok(sep) = tokens.curr(false) else {
                break;
            };
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                continue;
            }
            break;
        }

        Ok(bindings)
    }

    pub(super) fn parse_rolling_binding(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<RollingBinding, Box<dyn Glitch>> {
        let name_token = tokens.curr(false)?;
        let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &name_token,
                "Expected rolling binding name".to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let type_hint = if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                Some(self.parse_type_reference_tokens(tokens)?)
            } else {
                None
            }
        } else {
            None
        };

        self.skip_ignorable(tokens);
        let in_token = tokens.curr(false)?;
        if !matches!(in_token.key(), KEYWORD::Keyword(BUILDIN::In)) {
            return Err(Box::new(ParseError::from_token(
                &in_token,
                "Expected 'in' in rolling binding".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let iterable = self.parse_logical_expression(tokens)?;
        Ok(RollingBinding {
            name,
            type_hint,
            iterable,
        })
    }
}
