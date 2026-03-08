use super::*;

impl AstParser {
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

            let rhs = self.parse_logical_or_expression(tokens)?;
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
