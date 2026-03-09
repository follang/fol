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
        if self.lookahead_binding_alternative(tokens).is_some() {
            let nodes = self.parse_binding_alternative_decl(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(nodes);
        }

        if matches!(key, KEYWORD::Keyword(BUILDIN::Var)) {
            let nodes = self.parse_var_decl(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(nodes);
        }

        if matches!(key, KEYWORD::Keyword(BUILDIN::Let)) {
            let nodes = self.parse_let_decl(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(nodes);
        }

        if matches!(key, KEYWORD::Keyword(BUILDIN::Con)) {
            let nodes = self.parse_con_decl(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(nodes);
        }

        if matches!(key, KEYWORD::Keyword(BUILDIN::Lab)) {
            let nodes = self.parse_lab_decl(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(nodes);
        }

        if matches!(key, KEYWORD::Keyword(BUILDIN::Use)) {
            let nodes = self.parse_use_decl(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(nodes);
        }

        if matches!(key, KEYWORD::Symbol(SYMBOL::CurlyO)) {
            let node = self.parse_block_stmt(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(vec![node]);
        }

        if matches!(
            key,
            KEYWORD::Keyword(BUILDIN::Panic)
                | KEYWORD::Keyword(BUILDIN::Report)
                | KEYWORD::Keyword(BUILDIN::Check)
                | KEYWORD::Keyword(BUILDIN::Assert)
        ) {
            let node = self.parse_builtin_call_stmt(tokens)?;
            self.consume_optional_semicolon(tokens);
            return Ok(vec![node]);
        }

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
