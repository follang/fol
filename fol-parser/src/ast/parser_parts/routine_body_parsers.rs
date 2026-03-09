use super::*;

impl AstParser {
    fn parse_trailing_inquiries(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut inquiries = Vec::new();
        let mut inquiry_targets = HashSet::new();

        loop {
            self.skip_ignorable(tokens);
            let parsed = self.parse_optional_inquiry_clause(tokens)?;
            if parsed.is_empty() {
                break;
            }

            for inquiry in parsed {
                let target = match &inquiry {
                    AstNode::Inquiry { target, .. } => target.clone(),
                    _ => String::new(),
                };
                if !inquiry_targets.insert(target.clone()) {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        format!("Duplicate inquiry clause for '{}'", target),
                    )));
                }
                inquiries.push(inquiry);
            }
        }

        Ok(inquiries)
    }

    pub(super) fn parse_named_routine_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        open_body_message: &str,
        missing_close_message: &str,
    ) -> Result<(Vec<AstNode>, Vec<AstNode>), Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open_body = tokens.curr(false)?;

        if matches!(open_body.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            let _ = tokens.bump();
            let (body, mut inquiries) =
                self.parse_routine_body_with_inquiries(tokens, missing_close_message)?;
            inquiries.extend(self.parse_trailing_inquiries(tokens)?);
            return Ok((body, inquiries));
        }

        if matches!(open_body.key(), KEYWORD::Operator(OPERATOR::Flow)) {
            let body = self.parse_flow_body_nodes(tokens)?;
            let inquiries = self.parse_trailing_inquiries(tokens)?;
            return Ok((body, inquiries));
        }

        Err(Box::new(ParseError::from_token(
            &open_body,
            open_body_message.to_string(),
        )))
    }
}
