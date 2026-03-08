use super::*;

impl AstParser {
    pub(super) fn token_can_start_path_expression(
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> bool {
        if Self::token_to_named_label(token).is_none() {
            return false;
        }

        let text = token.con().trim();
        let numeric = text
            .strip_prefix(['+', '-'])
            .unwrap_or(text)
            .chars()
            .all(|c| c.is_ascii_digit());

        !numeric
    }

    fn consume_slice_separator(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Option<bool>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let token = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(None),
        };

        if matches!(token.key(), KEYWORD::Operator(OPERATOR::Path)) {
            let _ = tokens.bump();
            return Ok(Some(true));
        }

        if !matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Ok(None);
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        if matches!(
            tokens.curr(false).map(|token| token.key()),
            Ok(KEYWORD::Symbol(SYMBOL::Colon) | KEYWORD::Operator(OPERATOR::Path))
        ) {
            let _ = tokens.bump();
            return Ok(Some(true));
        }

        Ok(Some(false))
    }

    fn parse_optional_slice_end(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Option<Box<AstNode>>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        if matches!(
            tokens.curr(false).map(|token| token.key()),
            Ok(KEYWORD::Symbol(SYMBOL::SquarC))
        ) {
            Ok(None)
        } else {
            Ok(Some(Box::new(self.parse_logical_expression(tokens)?)))
        }
    }

    pub(super) fn parse_named_path(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        expected_root_error: &str,
        expected_segment_error: &str,
    ) -> Result<String, Box<dyn Glitch>> {
        let root = tokens.curr(false)?;
        let mut name = Self::token_to_named_label(&root).ok_or_else(|| {
            Box::new(ParseError::from_token(&root, expected_root_error.to_string())) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();

        loop {
            self.skip_ignorable(tokens);
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if !matches!(token.key(), KEYWORD::Operator(OPERATOR::Path)) {
                break;
            }

            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let segment = tokens.curr(false)?;
            let segment_name = Self::token_to_named_label(&segment).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &segment,
                    expected_segment_error.to_string(),
                )) as Box<dyn Glitch>
            })?;
            name.push_str("::");
            name.push_str(&segment_name);
            let _ = tokens.bump();
        }

        Ok(name)
    }

    pub(super) fn parse_index_or_slice_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        node: AstNode,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        if let Some(reverse) = self.consume_slice_separator(tokens)? {
            let end = self.parse_optional_slice_end(tokens)?;
            self.skip_ignorable(tokens);

            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected closing ']' for slice expression".to_string(),
                )));
            }

            let _ = tokens.bump();
            return Ok(AstNode::SliceAccess {
                container: Box::new(node),
                start: None,
                end,
                reverse,
            });
        }

        let start = self.parse_logical_expression(tokens)?;
        self.skip_ignorable(tokens);

        if let Some(reverse) = self.consume_slice_separator(tokens)? {
            let end = self.parse_optional_slice_end(tokens)?;
            self.skip_ignorable(tokens);

            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected closing ']' for slice expression".to_string(),
                )));
            }

            let _ = tokens.bump();
            return Ok(AstNode::SliceAccess {
                container: Box::new(node),
                start: Some(Box::new(start)),
                end,
                reverse,
            });
        }

        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected closing ']' for index expression".to_string(),
            )));
        }

        let _ = tokens.bump();
        Ok(AstNode::IndexAccess {
            container: Box::new(node),
            index: Box::new(start),
        })
    }

    pub(super) fn parse_index_or_slice_assignment_target(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        node: AstNode,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        if let Some(reverse) = self.consume_slice_separator(tokens)? {
            let end = self.parse_optional_slice_end(tokens)?;
            self.skip_ignorable(tokens);

            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected closing ']' for slice assignment target".to_string(),
                )));
            }

            let _ = tokens.bump();
            return Ok(AstNode::SliceAccess {
                container: Box::new(node),
                start: None,
                end,
                reverse,
            });
        }

        let start = self.parse_logical_expression(tokens)?;
        self.skip_ignorable(tokens);

        if let Some(reverse) = self.consume_slice_separator(tokens)? {
            let end = self.parse_optional_slice_end(tokens)?;
            self.skip_ignorable(tokens);

            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected closing ']' for slice assignment target".to_string(),
                )));
            }

            let _ = tokens.bump();
            return Ok(AstNode::SliceAccess {
                container: Box::new(node),
                start: Some(Box::new(start)),
                end,
                reverse,
            });
        }

        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected closing ']' for index assignment target".to_string(),
            )));
        }

        let _ = tokens.bump();
        Ok(AstNode::IndexAccess {
            container: Box::new(node),
            index: Box::new(start),
        })
    }
}
