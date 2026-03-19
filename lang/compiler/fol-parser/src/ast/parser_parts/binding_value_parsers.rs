use super::*;

impl AstParser {
    pub(super) fn parse_binding_values(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        allow_segment_break: bool,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut values = Vec::new();

        for _ in 0..256 {
            values.push(self.parse_logical_expression(tokens)?);
            self.skip_layout(tokens)?;

            let next = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                if allow_segment_break && self.lookahead_closes_binding_values(tokens) {
                    break;
                }
                if allow_segment_break && self.lookahead_starts_binding_segment(tokens) {
                    break;
                }
                let _ = tokens.bump();
                self.skip_ignorable(tokens)?;
                continue;
            }

            break;
        }

        Ok(values)
    }

    pub(super) fn lookahead_closes_binding_values(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        matches!(
            self.next_significant_key_from_window(tokens),
            Some(KEYWORD::Symbol(SYMBOL::RoundC))
                | Some(KEYWORD::Symbol(SYMBOL::Semi))
                | Some(KEYWORD::Symbol(SYMBOL::CurlyC))
        )
    }

    pub(super) fn lookahead_starts_binding_segment(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let mut iter = tokens
            .next_vec()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|token| {
                let key = token.key();
                !Self::key_is_soft_ignorable(&key)
            });

        let first = match iter.next() {
            Some(token) => token,
            None => return false,
        };
        if !(Self::token_to_named_label(&first).is_some() || first.key().is_illegal()) {
            return false;
        }

        matches!(
            iter.next(),
            Some(token)
                if matches!(
                    token.key(),
                    KEYWORD::Symbol(SYMBOL::Colon)
                        | KEYWORD::Symbol(SYMBOL::Equal)
                        | KEYWORD::Symbol(SYMBOL::Comma)
                )
        )
    }
}
