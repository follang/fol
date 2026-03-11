use super::*;

impl AstParser {
    pub(super) fn parse_test_type_arguments(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<(Option<String>, Vec<String>), Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '[' to start tst[...] arguments".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut values = Vec::new();
        for _ in 0..64 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                break;
            }

            let is_string = token.key().is_textual_literal();
            if is_string && !values.is_empty() {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Quoted tst[...] arguments are only allowed for the optional test label"
                        .to_string(),
                )));
            }

            let value = match token.key() {
                KEYWORD::Literal(LITERAL::Stringy) | KEYWORD::Literal(LITERAL::Quoted) => {
                    Self::exact_unquote_text(token.con())
                }
                _ => Self::token_to_named_label(&token).ok_or_else(|| {
                    Box::new(ParseError::from_token(
                        &token,
                        "Expected tst[...] argument".to_string(),
                    )) as Box<dyn Glitch>
                })?,
            };
            values.push((is_string, value));
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(
                sep.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                if matches!(
                    tokens.curr(false).map(|token| token.key()),
                    Ok(KEYWORD::Symbol(SYMBOL::SquarC))
                ) {
                    let _ = tokens.bump();
                    break;
                }
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                break;
            }

            if Self::is_missing_type_reference_close_token(&sep.key()) {
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected closing ']' in type reference".to_string(),
                )));
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ']' in tst[...] arguments".to_string(),
            )));
        }

        let (name, access) = match values.first() {
            Some((true, label)) => (
                Some(label.clone()),
                values.iter().skip(1).map(|(_, value)| value.clone()).collect(),
            ),
            _ => (None, values.into_iter().map(|(_, value)| value).collect()),
        };

        Ok((name, access))
    }
}
