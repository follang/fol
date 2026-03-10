use super::*;

impl AstParser {
    pub(super) fn parse_postfix_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        mut node: AstNode,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        for _ in 0..256 {
            self.skip_ignorable(tokens);

            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(node),
            };

            match token.key() {
                KEYWORD::Symbol(SYMBOL::RoundO) => {
                    let _ = tokens.bump();
                    let args = self.parse_call_args(tokens)?;
                    node = match node {
                        AstNode::Identifier { name } => AstNode::FunctionCall { name, args },
                        callee => AstNode::Invoke {
                            callee: Box::new(callee),
                            args,
                        },
                    };
                }
                KEYWORD::Symbol(SYMBOL::Dot) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);

                    let member_token = tokens.curr(false)?;
                    let member = Self::token_to_named_label(&member_token).ok_or_else(|| {
                        Box::new(ParseError::from_token(
                            &member_token,
                            "Expected field or method name after '.'".to_string(),
                        )) as Box<dyn Glitch>
                    })?;
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);

                    let is_method_call = matches!(
                        tokens.curr(false).map(|token| token.key()),
                        Ok(KEYWORD::Symbol(SYMBOL::RoundO))
                    );

                    if is_method_call {
                        let _ = tokens.bump();
                        let args = self.parse_call_args(tokens)?;
                        node = AstNode::MethodCall {
                            object: Box::new(node),
                            method: member,
                            args,
                        };
                    } else {
                        node = AstNode::FieldAccess {
                            object: Box::new(node),
                            field: member,
                        };
                    }
                }
                KEYWORD::Symbol(SYMBOL::SquarO) => {
                    node = self.parse_index_or_slice_expression(tokens, node)?;
                }
                KEYWORD::Symbol(SYMBOL::Colon) => {
                    let next_key = self.next_significant_key_from_window(tokens);
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::SquarO))) {
                        node = self.parse_prefix_availability_expression(tokens, node)?;
                    } else if matches!(
                        node,
                        AstNode::IndexAccess { .. }
                            | AstNode::SliceAccess { .. }
                            | AstNode::PatternAccess { .. }
                    ) {
                        let _ = tokens.bump();
                        node = AstNode::AvailabilityAccess {
                            target: Box::new(node),
                        };
                    } else {
                        break;
                    }
                }
                KEYWORD::Symbol(SYMBOL::Bang) => {
                    if matches!(
                        self.next_significant_key_from_window(tokens),
                        Some(KEYWORD::Symbol(SYMBOL::Equal))
                    ) {
                        break;
                    }

                    let _ = tokens.bump();
                    node = AstNode::UnaryOp {
                        op: UnaryOperator::Unwrap,
                        operand: Box::new(node),
                    };
                }
                KEYWORD::Symbol(SYMBOL::Dollar) => {
                    let _ = tokens.bump();
                    node = AstNode::TemplateCall {
                        object: Box::new(node),
                        template: "$".to_string(),
                    };
                }
                _ => break,
            }
        }

        Ok(node)
    }
}
