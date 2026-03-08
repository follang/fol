use super::*;

impl AstParser {
    pub(super) fn parse_pipe_lambda_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '|' to start lambda expression".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut params = Vec::new();
        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
            loop {
                let name_token = tokens.curr(false)?;
                let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
                    Box::new(ParseError::from_token(
                        &name_token,
                        "Expected lambda parameter name".to_string(),
                    )) as Box<dyn Glitch>
                })?;
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                let mut param_type = FolType::Any;
                if let Ok(token) = tokens.curr(false) {
                    if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        param_type = self.parse_type_reference_tokens(tokens)?;
                    }
                }
                params.push(Parameter {
                    name,
                    param_type,
                    is_borrowable: false,
                    default: None,
                });

                self.skip_ignorable(tokens);
                let token = tokens.curr(false)?;
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    continue;
                }
                break;
            }
        }

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::Pipe)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected closing '|' after lambda parameters".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let captures = self.parse_optional_routine_capture_list(tokens)?;
        self.ensure_unique_capture_names(&captures)?;

        self.skip_ignorable(tokens);
        let body = if matches!(tokens.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            let _ = tokens.bump();
            self.parse_block_body(tokens, "Expected '}' to close lambda body")?
        } else {
            vec![AstNode::Return {
                value: Some(Box::new(self.parse_logical_expression(tokens)?)),
            }]
        };

        Ok(AstNode::AnonymousFun {
            options: vec![FunOption::Mutable],
            captures,
            params,
            return_type: None,
            error_type: None,
            body,
            inquiries: Vec::new(),
        })
    }
}
