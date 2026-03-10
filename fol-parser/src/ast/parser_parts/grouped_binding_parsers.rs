use super::*;
use crate::ast::BindingPattern;

impl AstParser {
    pub(super) fn parse_binding_group(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        options: Vec<VarOption>,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '(' to start grouped bindings".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut nodes = Vec::new();
        for _ in 0..512 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                self.consume_optional_semicolon(tokens);
                return Ok(nodes);
            }

            let patterns = self.parse_binding_pattern_list(tokens, "grouped binding")?;
            let is_destructuring = patterns.iter().any(BindingPattern::is_destructuring);
            self.skip_ignorable(tokens);

            let mut type_hint = None;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    type_hint = Some(self.parse_type_reference_tokens(tokens)?);
                }
            }

            self.skip_ignorable(tokens);
            let mut values = Vec::new();
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    values = self.parse_binding_values(tokens, !is_destructuring)?;
                }
            }

            if is_destructuring {
                nodes.push(self.build_destructure_binding_node(
                    options.clone(),
                    patterns,
                    type_hint,
                    values,
                )?);
            } else {
                let names = patterns
                    .into_iter()
                    .map(|pattern| match pattern {
                        BindingPattern::Name(name) => Ok(name),
                        other => Err(Box::new(ParseError {
                            message: format!("Unsupported grouped binding pattern: {other:?}"),
                            file: None,
                            line: 0,
                            column: 0,
                            length: 0,
                        }) as Box<dyn Glitch>),
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                nodes.extend(self.build_binding_nodes(options.clone(), names, type_hint, values)?);
            }

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                self.consume_optional_semicolon(tokens);
                return Ok(nodes);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or ')' in grouped bindings".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Grouped bindings exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }
}
