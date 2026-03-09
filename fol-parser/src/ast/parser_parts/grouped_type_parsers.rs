use super::*;

impl AstParser {
    pub(super) fn parse_type_group(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        options: Vec<TypeOption>,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '(' to start grouped type declarations".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut nodes = Vec::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                self.consume_optional_semicolon(tokens);
                return Ok(nodes);
            }

            nodes.push(self.parse_single_type_decl_with_options(tokens, options.clone(), false)?);

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(
                sep.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                self.consume_optional_semicolon(tokens);
                return Ok(nodes);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ')' in grouped type declarations".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Grouped type declarations exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }
}
