use super::*;

impl AstParser {
    pub(super) fn flow_nodes_to_expr(&self, nodes: Vec<AstNode>) -> AstNode {
        if nodes.len() == 1 {
            nodes.into_iter().next().expect("one node")
        } else {
            AstNode::Block { statements: nodes }
        }
    }

    pub(super) fn parse_flow_body_nodes(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let flow = tokens.curr(false)?;
        if !matches!(flow.key(), KEYWORD::Operator(OPERATOR::Flow)) {
            return Err(Box::new(ParseError::from_token(
                &flow,
                "Expected '=>' to start flow body".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let key = tokens.curr(false)?.key();
        let node = if (AstParser::token_can_be_logical_name(&key)
            || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
            && self.lookahead_is_assignment(tokens)
        {
            self.parse_assignment_stmt(tokens)?
        } else if (AstParser::token_can_be_logical_name(&key)
            || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
            && (self.lookahead_is_call(tokens) || self.lookahead_is_method_call(tokens))
            && self.can_start_assignment(tokens)
        {
            self.parse_call_stmt(tokens)?
        } else if (matches!(key, KEYWORD::Symbol(SYMBOL::RoundO))
            || AstParser::token_can_be_logical_name(&key)
            || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
            && self.lookahead_is_general_invoke(tokens, matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)))
            && self.can_start_assignment(tokens)
        {
            self.parse_invoke_stmt(tokens)?
        } else {
            self.parse_logical_expression(tokens)?
        };

        self.consume_optional_semicolon(tokens);
        Ok(vec![node])
    }
}
