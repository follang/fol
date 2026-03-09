use super::*;

impl AstParser {
    fn lookahead_pipe_stage_has_body_after_parens(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let mut depth = 0usize;
        let mut saw_open = false;

        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };
            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            match key {
                KEYWORD::Symbol(SYMBOL::RoundO) => {
                    saw_open = true;
                    depth += 1;
                }
                KEYWORD::Symbol(SYMBOL::RoundC) => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                    if saw_open && depth == 0 {
                        continue;
                    }
                }
                KEYWORD::Symbol(SYMBOL::CurlyO) | KEYWORD::Operator(OPERATOR::Flow)
                    if saw_open && depth == 0 =>
                {
                    return true;
                }
                _ if saw_open && depth == 0 => {
                    return false;
                }
                _ => {}
            }
        }

        false
    }

    fn parse_pipe_stage_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let token = tokens.curr(false)?;

        if matches!(token.key(), KEYWORD::Keyword(BUILDIN::If))
            && self.lookahead_pipe_stage_has_body_after_parens(tokens)
        {
            return self.parse_if_stmt(tokens);
        }

        if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Return)) {
            return self.parse_return_stmt(tokens);
        }

        if matches!(
            token.key(),
            KEYWORD::Keyword(BUILDIN::Panic)
                | KEYWORD::Keyword(BUILDIN::Report)
                | KEYWORD::Keyword(BUILDIN::Check)
                | KEYWORD::Keyword(BUILDIN::Assert)
        ) {
            if matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Symbol(SYMBOL::RoundO))
            ) {
                return self.parse_logical_or_expression(tokens);
            }
            return self.parse_builtin_call_stmt(tokens);
        }

        self.parse_logical_or_expression(tokens)
    }

    pub(super) fn parse_pipe_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_logical_or_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);
            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            if !matches!(op_token.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
                break;
            }

            let consume_count = if matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Symbol(SYMBOL::Pipe))
            ) {
                2
            } else {
                1
            };

            for _ in 0..consume_count {
                self.consume_significant_token(tokens);
            }

            self.skip_ignorable(tokens);
            let next = tokens.curr(false)?;
            if next.key().is_terminal() || next.key().is_eof() {
                return Err(Box::new(ParseError::from_token(
                    &next,
                    if consume_count == 2 {
                        "Expected expression after '||'".to_string()
                    } else {
                        "Expected expression after '|'".to_string()
                    },
                )));
            }

            let rhs = self.parse_pipe_stage_expression(tokens)?;
            lhs = AstNode::BinaryOp {
                op: if consume_count == 2 {
                    BinaryOperator::PipeOr
                } else {
                    BinaryOperator::Pipe
                },
                left: Box::new(lhs),
                right: Box::new(rhs),
            };
        }

        Ok(lhs)
    }
}
