use super::*;

impl AstParser {
    pub(super) fn parse_binding_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        default_options: Vec<VarOption>,
    ) -> Result<Vec<VarOption>, Box<dyn Glitch>> {
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(default_options),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(default_options);
        }
        let _ = tokens.bump();

        let mut parsed_options = Vec::new();
        for _ in 0..16 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(self.merge_binding_options(default_options, parsed_options));
            }

            let option = match token.con().trim() {
                "mut" | "mutable" => VarOption::Mutable,
                "imu" | "immutable" => VarOption::Immutable,
                _ => {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Unknown binding option".to_string(),
                    )));
                }
            };
            parsed_options.push(option);
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(self.merge_binding_options(default_options, parsed_options));
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or ']' in binding options".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Binding options exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }

    pub(super) fn merge_binding_options(
        &self,
        mut base: Vec<VarOption>,
        parsed: Vec<VarOption>,
    ) -> Vec<VarOption> {
        for option in parsed {
            match option {
                VarOption::Mutable | VarOption::Immutable => {
                    base.retain(|existing| {
                        !matches!(existing, VarOption::Mutable | VarOption::Immutable)
                    });
                }
                VarOption::Export | VarOption::Hidden | VarOption::Normal => {
                    base.retain(|existing| {
                        !matches!(
                            existing,
                            VarOption::Export | VarOption::Hidden | VarOption::Normal
                        )
                    });
                }
                _ => {}
            }

            if !base.contains(&option) {
                base.push(option);
            }
        }

        base
    }
}
