use super::*;

impl AstParser {
    pub(super) fn validate_decl_visibility_options(
        &self,
        options: &[DeclOption],
        context: &str,
    ) -> Result<(), Box<dyn Glitch>> {
        let mut saw_export = false;
        let mut saw_hidden = false;
        let mut saw_normal = false;

        for option in options {
            match option {
                DeclOption::Export => {
                    if saw_export {
                        return Err(Box::new(ParseError {
                            kind: ParseErrorKind::Syntax,
                            message: format!("Duplicate {} option 'export'", context),
                            file: None,
                            line: 1,
                            column: 1,
                            length: 1,
                        }));
                    }
                    saw_export = true;
                }
                DeclOption::Hidden => {
                    if saw_hidden {
                        return Err(Box::new(ParseError {
                            kind: ParseErrorKind::Syntax,
                            message: format!("Duplicate {} option 'hidden'", context),
                            file: None,
                            line: 1,
                            column: 1,
                            length: 1,
                        }));
                    }
                    saw_hidden = true;
                }
                DeclOption::Normal => {
                    if saw_normal {
                        return Err(Box::new(ParseError {
                            kind: ParseErrorKind::Syntax,
                            message: format!("Duplicate {} option 'normal'", context),
                            file: None,
                            line: 1,
                            column: 1,
                            length: 1,
                        }));
                    }
                    saw_normal = true;
                }
            }
        }

        if (saw_export as u8 + saw_hidden as u8 + saw_normal as u8) > 1 {
            return Err(Box::new(ParseError {
                kind: ParseErrorKind::Syntax,
                message: format!("Conflicting {} visibility options", context),
                file: None,
                line: 1,
                column: 1,
                length: 1,
            }));
        }

        Ok(())
    }

    pub(super) fn parse_decl_visibility_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        context: &str,
    ) -> Result<Vec<DeclOption>, Box<dyn Glitch>> {
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(Vec::new()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(Vec::new());
        }
        let _ = tokens.bump();

        let mut options = Vec::new();
        for _ in 0..16 {
            self.skip_ignorable(tokens)?;
            let token = tokens.curr(false)?;
            Self::reject_illegal_token(&token)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                self.validate_decl_visibility_options(&options, context)?;
                return Ok(options);
            }

            let option = match token.con().trim() {
                "+" | "pub" | "exp" | "export" => DeclOption::Export,
                "-" | "hid" | "hidden" => DeclOption::Hidden,
                "nor" | "normal" => DeclOption::Normal,
                _ => {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        format!("Unknown {} option", context),
                    )))
                }
            };
            options.push(option);
            let _ = tokens.bump();

            self.skip_ignorable(tokens)?;
            let sep = tokens.curr(false)?;
            Self::reject_illegal_token(&sep)?;
            if matches!(
                sep.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens)?;
                if matches!(
                    tokens.curr(false).map(|token| token.key()),
                    Ok(KEYWORD::Symbol(SYMBOL::SquarC))
                ) {
                    let _ = tokens.bump();
                    self.validate_decl_visibility_options(&options, context)?;
                    return Ok(options);
                }
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                self.validate_decl_visibility_options(&options, context)?;
                return Ok(options);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ']' in declaration options".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            kind: ParseErrorKind::Syntax,
            message: "Declaration options exceeded parser limit".to_string(),
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }))
    }
}
