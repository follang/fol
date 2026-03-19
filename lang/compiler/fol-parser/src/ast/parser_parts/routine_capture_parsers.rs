use super::*;

impl AstParser {
    pub(super) fn ensure_unique_capture_names(
        &self,
        captures: &[String],
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> Result<(), Box<dyn Glitch>> {
        let mut seen = HashSet::new();
        for capture in captures {
            if !seen.insert(canonical_identifier_key(capture)) {
                let error = if let Ok(token) = tokens.curr(false) {
                    ParseError::from_token(
                        &token,
                        format!("Duplicate capture name '{}'", capture),
                    )
                } else {
                    ParseError {
                        kind: ParseErrorKind::Syntax,
                        message: format!("Duplicate capture name '{}'", capture),
                        file: None,
                        line: 0,
                        column: 0,
                        length: 0,
                    }
                };
                return Err(Box::new(error));
            }
        }

        Ok(())
    }

    pub(super) fn parse_optional_routine_capture_list(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<String>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens)?;
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(Vec::new()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(Vec::new());
        }
        let _ = tokens.bump();

        let mut captures = Vec::new();
        for _ in 0..128 {
            self.skip_ignorable(tokens)?;
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(captures);
            }

            let name =
                Self::expect_named_label(&token, "Expected capture name in routine capture list")?;
            captures.push(name);
            let _ = tokens.bump();

            self.skip_ignorable(tokens)?;
            let sep = tokens.curr(false)?;
            if matches!(
                sep.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens)?;
                if matches!(
                    tokens.curr(false).map(|token| token.key()),
                    Ok(KEYWORD::Symbol(SYMBOL::SquarC))
                ) {
                    let _ = tokens.bump();
                    return Ok(captures);
                }
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(captures);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ']' in routine capture list".to_string(),
            )));
        }

        let error = if let Ok(token) = tokens.curr(false) {
            ParseError::from_token(
                &token,
                "Routine capture parsing exceeded safety bound".to_string(),
            )
        } else {
            ParseError {
                kind: ParseErrorKind::Syntax,
                message: "Routine capture parsing exceeded safety bound".to_string(),
                file: None,
                line: 0,
                column: 0,
                length: 0,
            }
        };
        Err(Box::new(error))
    }
}
