use super::*;

impl AstParser {
    pub(super) fn parse_entry_type_definition(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<TypeDefinition, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start type entry definition".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut variants = HashMap::new();
        let mut variant_meta = HashMap::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(TypeDefinition::Entry {
                    variants,
                    variant_meta,
                });
            }

            let default_options = match token.key() {
                KEYWORD::Keyword(BUILDIN::Var) => {
                    let _ = tokens.bump();
                    vec![VarOption::Mutable, VarOption::Normal]
                }
                KEYWORD::Keyword(BUILDIN::Lab) => {
                    let _ = tokens.bump();
                    vec![VarOption::Immutable, VarOption::Normal]
                }
                KEYWORD::Keyword(BUILDIN::Con) => {
                    let _ = tokens.bump();
                    vec![VarOption::Immutable, VarOption::Normal]
                }
                _ => {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected 'var', 'lab', or 'con' in type entry definition".to_string(),
                    )))
                }
            };

            let mut names = Vec::new();
            for _ in 0..64 {
                self.skip_ignorable(tokens);
                let name_token = tokens.curr(false)?;
                let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
                    Box::new(ParseError::from_token(
                        &name_token,
                        "Expected entry variant name".to_string(),
                    )) as Box<dyn Glitch>
                })?;
                names.push(name);
                let _ = tokens.bump();

                self.skip_ignorable(tokens);
                let sep = tokens.curr(false)?;
                if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                    let _ = tokens.bump();
                    continue;
                }
                break;
            }

            self.skip_ignorable(tokens);
            let mut variant_type = None;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    variant_type = Some(self.parse_type_reference_tokens(tokens)?);
                }
            }

            self.skip_ignorable(tokens);
            let mut default = None;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    default = Some(self.parse_logical_expression(tokens)?);
                }
            }

            for name in names {
                if variants.insert(name.clone(), variant_type.clone()).is_some() {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        format!("Duplicate entry variant '{}'", name),
                    )));
                }
                variant_meta.insert(
                    name,
                    EntryVariantMeta {
                        default: default.clone(),
                        options: default_options.clone(),
                    },
                );
            }

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma))
                || matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Semi))
            {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(TypeDefinition::Entry {
                    variants,
                    variant_meta,
                });
            }
            if sep.key().is_terminal() || matches!(sep.key(), KEYWORD::Void(_)) {
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected '}' to close type entry definition".to_string(),
                )));
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or '}' in type entry definition".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Type entry definition exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }

    pub(super) fn parse_record_type_definition(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<TypeDefinition, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start type record definition".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut fields = HashMap::new();
        let mut field_meta = HashMap::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(TypeDefinition::Record { fields, field_meta });
            }

            if token.key().is_terminal() || matches!(token.key(), KEYWORD::Void(_)) {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Expected '}' to close type record definition".to_string(),
                )));
            }

            let options = match token.key() {
                KEYWORD::Keyword(BUILDIN::Var) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    vec![VarOption::Mutable, VarOption::Normal]
                }
                KEYWORD::Keyword(BUILDIN::Lab) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    vec![VarOption::Immutable, VarOption::Normal]
                }
                KEYWORD::Keyword(BUILDIN::Con) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    vec![VarOption::Immutable, VarOption::Normal]
                }
                _ => Vec::new(),
            };

            let token = tokens.curr(false)?;
            let field_name = Self::token_to_named_label(&token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &token,
                    "Expected field name in type record definition".to_string(),
                )) as Box<dyn Glitch>
            })?;
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let colon = tokens.curr(false)?;
            if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                return Err(Box::new(ParseError::from_token(
                    &colon,
                    "Expected ':' after record field name".to_string(),
                )));
            }
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let field_type = self.parse_type_reference_tokens(tokens)?;
            if fields.insert(field_name.clone(), field_type).is_some() {
                let token = tokens.curr(false)?;
                return Err(Box::new(ParseError::from_token(
                    &token,
                    format!("Duplicate record field '{}'", field_name),
                )));
            }

            self.skip_ignorable(tokens);
            let mut default = None;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                    default = Some(self.parse_logical_expression(tokens)?);
                    self.skip_ignorable(tokens);
                }
            }

            field_meta.insert(field_name, RecordFieldMeta { default, options });

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma))
                || matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Semi))
            {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(TypeDefinition::Record { fields, field_meta });
            }
            if sep.key().is_terminal() || matches!(sep.key(), KEYWORD::Void(_)) {
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected '}' to close type record definition".to_string(),
                )));
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or '}' in type record definition".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Type record definition exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }
}
