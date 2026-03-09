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
        let mut members = Vec::new();
        let mut seen_members = HashSet::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(TypeDefinition::Entry {
                    variants,
                    variant_meta,
                    members,
                });
            }

            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Pro)
                    | KEYWORD::Keyword(BUILDIN::Log)
            ) {
                let member = self.parse_standard_routine_signature(tokens)?;
                let key = self.type_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_type_member_error(&token, &key));
                }
                members.push(member);
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
                        members,
                    });
                }
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected ',', ';', or '}' in type entry definition".to_string(),
                )));
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Ali)) {
                let member = self.parse_alias_decl(tokens)?;
                let key = self.type_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_type_member_error(&token, &key));
                }
                members.push(member);
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
                        members,
                    });
                }
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected ',', ';', or '}' in type entry definition".to_string(),
                )));
            }

            let default_options = if let Some((keyword, options)) =
                self.lookahead_binding_alternative(tokens)
            {
                match keyword {
                    "var" | "con" => {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        options
                    }
                    _ => {
                        return Err(Box::new(ParseError::from_token(
                            &token,
                            "Expected 'var', 'lab', or 'con' in type entry definition"
                                .to_string(),
                        )))
                    }
                }
            } else {
                match token.key() {
                KEYWORD::Keyword(BUILDIN::Var) => {
                    let _ = tokens.bump();
                    self.parse_binding_options(
                        tokens,
                        vec![VarOption::Mutable, VarOption::Normal],
                    )?
                }
                KEYWORD::Keyword(BUILDIN::Lab) => {
                    let _ = tokens.bump();
                    self.parse_binding_options(
                        tokens,
                        vec![VarOption::Immutable, VarOption::Normal],
                    )?
                }
                KEYWORD::Keyword(BUILDIN::Con) => {
                    let _ = tokens.bump();
                    self.parse_binding_options(
                        tokens,
                        vec![VarOption::Immutable, VarOption::Normal],
                    )?
                }
                _ => {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected 'var', 'lab', or 'con' in type entry definition".to_string(),
                    )))
                }
                }
            };

            self.skip_ignorable(tokens);

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
                names.push((name, name_token));
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

            for (name, name_token) in names {
                if variants.insert(name.clone(), variant_type.clone()).is_some() {
                    return Err(Box::new(ParseError::from_token(
                        &name_token,
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
                    members,
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
        let mut members = Vec::new();
        let mut seen_members = HashSet::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(TypeDefinition::Record {
                    fields,
                    field_meta,
                    members,
                });
            }

            if token.key().is_terminal() || matches!(token.key(), KEYWORD::Void(_)) {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Expected '}' to close type record definition".to_string(),
                )));
            }

            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Pro)
                    | KEYWORD::Keyword(BUILDIN::Log)
            ) {
                let member = self.parse_standard_routine_signature(tokens)?;
                let key = self.type_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_type_member_error(&token, &key));
                }
                members.push(member);
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
                    return Ok(TypeDefinition::Record {
                        fields,
                        field_meta,
                        members,
                    });
                }
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected ',', ';', or '}' in type record definition".to_string(),
                )));
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Ali)) {
                let member = self.parse_alias_decl(tokens)?;
                let key = self.type_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_type_member_error(&token, &key));
                }
                members.push(member);
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
                    return Ok(TypeDefinition::Record {
                        fields,
                        field_meta,
                        members,
                    });
                }
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected ',', ';', or '}' in type record definition".to_string(),
                )));
            }

            let options = if let Some((keyword, options)) = self.lookahead_binding_alternative(tokens)
            {
                match keyword {
                    "var" | "con" => {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        options
                    }
                    _ => Vec::new(),
                }
            } else {
                match token.key() {
                KEYWORD::Keyword(BUILDIN::Var) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    self.parse_binding_options(
                        tokens,
                        vec![VarOption::Mutable, VarOption::Normal],
                    )?
                }
                KEYWORD::Keyword(BUILDIN::Lab) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    self.parse_binding_options(
                        tokens,
                        vec![VarOption::Immutable, VarOption::Normal],
                    )?
                }
                KEYWORD::Keyword(BUILDIN::Con) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    self.parse_binding_options(
                        tokens,
                        vec![VarOption::Immutable, VarOption::Normal],
                    )?
                }
                _ => Vec::new(),
                }
            };

            self.skip_ignorable(tokens);

            let mut field_names = Vec::new();
            loop {
                let name_token = tokens.curr(false)?;
                let field_name = Self::token_to_named_label(&name_token).ok_or_else(|| {
                    Box::new(ParseError::from_token(
                        &name_token,
                        "Expected field name in type record definition".to_string(),
                    )) as Box<dyn Glitch>
                })?;
                field_names.push((field_name, name_token));
                let _ = tokens.bump();

                self.skip_ignorable(tokens);
                let sep = tokens.curr(false)?;
                if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    continue;
                }
                break;
            }
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

            self.skip_ignorable(tokens);
            let mut default = None;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                    default = Some(self.parse_logical_expression(tokens)?);
                    self.skip_ignorable(tokens);
                }
            }

            for (field_name, name_token) in field_names {
                if fields
                    .insert(field_name.clone(), field_type.clone())
                    .is_some()
                {
                    return Err(Box::new(ParseError::from_token(
                        &name_token,
                        format!("Duplicate record field '{}'", field_name),
                    )));
                }
                field_meta.insert(
                    field_name,
                    RecordFieldMeta {
                        default: default.clone(),
                        options: options.clone(),
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
                return Ok(TypeDefinition::Record {
                    fields,
                    field_meta,
                    members,
                });
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

    fn type_member_key(&self, node: &AstNode) -> String {
        match node {
            AstNode::FunDecl { name, params, .. } | AstNode::ProDecl { name, params, .. } => {
                format!("{}#{}", name, params.len())
            }
            AstNode::AliasDecl { name, .. }
            | AstNode::TypeDecl { name, .. }
            | AstNode::VarDecl { name, .. }
            | AstNode::LabDecl { name, .. } => name.clone(),
            _ => String::new(),
        }
    }

    fn duplicate_type_member_error(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
        key: &str,
    ) -> Box<dyn Glitch> {
        Box::new(ParseError::from_token(
            token,
            format!("Duplicate type member '{}'", key),
        ))
    }
}
