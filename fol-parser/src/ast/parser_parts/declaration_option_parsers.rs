use super::*;

impl AstParser {
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
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
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

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(options);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or ']' in declaration options".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Declaration options exceeded parser limit".to_string(),
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }))
    }
}
