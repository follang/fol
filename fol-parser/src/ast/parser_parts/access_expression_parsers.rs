use super::*;

impl AstParser {
    pub(super) fn parse_index_or_slice_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        node: AstNode,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let next = tokens.curr(false)?;
        if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let end = if matches!(
                tokens.curr(false).map(|token| token.key()),
                Ok(KEYWORD::Symbol(SYMBOL::SquarC))
            ) {
                None
            } else {
                Some(Box::new(self.parse_logical_expression(tokens)?))
            };
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
                reverse: false,
            });
        }

        let start = self.parse_logical_expression(tokens)?;
        self.skip_ignorable(tokens);

        let next = tokens.curr(false)?;
        if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let end = if matches!(
                tokens.curr(false).map(|token| token.key()),
                Ok(KEYWORD::Symbol(SYMBOL::SquarC))
            ) {
                None
            } else {
                Some(Box::new(self.parse_logical_expression(tokens)?))
            };
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
                reverse: false,
            });
        }

        if !matches!(next.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &next,
                "Expected closing ']' for index expression".to_string(),
            )));
        }

        let _ = tokens.bump();
        Ok(AstNode::IndexAccess {
            container: Box::new(node),
            index: Box::new(start),
        })
    }
}
