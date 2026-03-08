use super::*;

impl AstParser {
    pub(super) fn parse_use_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let use_token = tokens.curr(false)?;
        if !matches!(use_token.key(), KEYWORD::Keyword(BUILDIN::Use)) {
            return Err(Box::new(ParseError::from_token(
                &use_token,
                "Expected 'use' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let options = self.parse_use_options(tokens)?;
        self.skip_ignorable(tokens);

        let names = self.parse_use_names(tokens)?;

        self.skip_ignorable(tokens);
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
            }
        }
        let path_type = self.parse_type_reference_tokens(tokens)?;

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' in use declaration".to_string(),
            )));
        }
        let _ = tokens.bump();

        let paths = self.parse_use_paths(tokens)?;
        self.consume_optional_semicolon(tokens);
        self.build_use_nodes(options, names, path_type, paths, &use_token)
    }

    fn parse_use_paths(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<String>, Box<dyn Glitch>> {
        let mut paths = Vec::new();

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let open = tokens.curr(false)?;
            if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                return Err(Box::new(ParseError::from_token(
                    &open,
                    "Expected '{' to start use path".to_string(),
                )));
            }
            let _ = tokens.bump();

            paths.push(self.parse_use_path(tokens)?);
            self.skip_ignorable(tokens);

            let next = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                continue;
            }

            break;
        }

        Ok(paths)
    }

    fn build_use_nodes(
        &self,
        options: Vec<UseOption>,
        names: Vec<String>,
        path_type: FolType,
        paths: Vec<String>,
        use_token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let assigned_paths = match paths.len() {
            1 => vec![paths[0].clone(); names.len()],
            n if n == names.len() => paths,
            _ => {
                return Err(Box::new(ParseError::from_token(
                    use_token,
                    "Use path count must match declared names or provide a single shared path"
                        .to_string(),
                )))
            }
        };

        Ok(names
            .into_iter()
            .zip(assigned_paths)
            .map(|(name, path)| AstNode::UseDecl {
                options: options.clone(),
                name,
                path_type: path_type.clone(),
                path,
            })
            .collect())
    }

    fn parse_use_names(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<String>, Box<dyn Glitch>> {
        let mut names = Vec::new();

        for _ in 0..256 {
            let name_token = tokens.curr(false)?;
            let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &name_token,
                    "Expected identifier after 'use'".to_string(),
                )) as Box<dyn Glitch>
            })?;

            names.push(name);
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let next = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                continue;
            }

            break;
        }

        Ok(names)
    }
}
