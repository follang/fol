use super::*;
use crate::ast::{UsePathSegment, UsePathSeparator};

#[derive(Debug, Clone)]
struct ParsedUsePath {
    segments: Vec<UsePathSegment>,
}

impl AstParser {
    pub(super) fn ensure_complete_use_path(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
        path: &str,
    ) -> Result<(), Box<dyn Glitch>> {
        if path.trim_end().ends_with("::") {
            return Err(Box::new(ParseError::from_token(
                token,
                "Expected name after '::' in use path".to_string(),
            )));
        }

        Ok(())
    }

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
    ) -> Result<Vec<ParsedUsePath>, Box<dyn Glitch>> {
        let mut paths = Vec::new();

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
                let _ = tokens.bump();
                let raw = self.parse_use_path(tokens)?;
                let segments = self.parse_use_path_segments(&raw, &token)?;
                paths.push(ParsedUsePath { segments });
            } else if token.key().is_textual_literal() {
                let raw = Self::exact_unquote_text(token.con());
                let segments = self.parse_use_path_segments(&raw, &token)?;
                paths.push(ParsedUsePath { segments });
                let _ = tokens.bump();
            } else {
                let raw = self.parse_direct_use_path(tokens)?;
                let segments = self.parse_use_path_segments(&raw, &token)?;
                paths.push(ParsedUsePath { segments });
            }
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
        paths: Vec<ParsedUsePath>,
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
                syntax_id: self.record_syntax_origin(use_token),
                options: options.clone(),
                name,
                path_type: path_type.clone(),
                path_segments: path.segments,
            })
            .collect())
    }

    fn parse_use_names(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<String>, Box<dyn Glitch>> {
        let mut names = Vec::new();
        let mut seen_names = HashSet::new();

        for _ in 0..256 {
            let name_token = tokens.curr(false)?;
            let name = Self::expect_named_label(&name_token, "Expected identifier after 'use'")?;
            if !seen_names.insert(canonical_identifier_key(&name)) {
                return Err(Box::new(ParseError::from_token(
                    &name_token,
                    format!("Duplicate use name '{}'", name),
                )));
            }

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

    fn parse_direct_use_path(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<String, Box<dyn Glitch>> {
        let mut path = String::new();

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            Self::reject_illegal_token(&token)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Comma))
                || matches!(token.key(), KEYWORD::Symbol(SYMBOL::Semi))
                || token.key().is_terminal()
                || matches!(token.key(), KEYWORD::Void(_))
            {
                break;
            }

            path.push_str(token.con().trim());
            if tokens.bump().is_none() {
                break;
            }
        }

        if path.is_empty() {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected use path".to_string(),
            )));
        }

        let token = tokens.curr(false)?;
        self.ensure_complete_use_path(&token, &path)?;
        Ok(path)
    }

    fn parse_use_path_segments(
        &self,
        path: &str,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<Vec<UsePathSegment>, Box<dyn Glitch>> {
        if path.contains("://") {
            return Ok(vec![UsePathSegment {
                separator: None,
                spelling: path.to_string(),
            }]);
        }

        let mut segments = Vec::new();
        let mut current = String::new();
        let mut pending_separator = None;
        let mut active_quote = None;
        let mut chars = path.chars().peekable();

        while let Some(ch) = chars.next() {
            if let Some(quote) = active_quote {
                current.push(ch);
                if ch == quote {
                    active_quote = None;
                }
                continue;
            }

            match ch {
                '\'' | '"' => {
                    active_quote = Some(ch);
                    current.push(ch);
                }
                '/' => {
                    self.finish_use_path_segment(
                        &mut segments,
                        &mut current,
                        &mut pending_separator,
                        token,
                    )?;
                    pending_separator = Some(UsePathSeparator::Slash);
                }
                ':' if matches!(chars.peek(), Some(':')) => {
                    let _ = chars.next();
                    self.finish_use_path_segment(
                        &mut segments,
                        &mut current,
                        &mut pending_separator,
                        token,
                    )?;
                    pending_separator = Some(UsePathSeparator::DoubleColon);
                }
                _ => current.push(ch),
            }
        }

        self.finish_use_path_segment(
            &mut segments,
            &mut current,
            &mut pending_separator,
            token,
        )?;

        Ok(segments)
    }

    fn finish_use_path_segment(
        &self,
        segments: &mut Vec<UsePathSegment>,
        current: &mut String,
        pending_separator: &mut Option<UsePathSeparator>,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<(), Box<dyn Glitch>> {
        if current.is_empty() {
            return Err(Box::new(ParseError::from_token(
                token,
                if pending_separator.is_some() {
                    "Expected use path segment after separator".to_string()
                } else {
                    "Expected use path segment".to_string()
                },
            )));
        }

        segments.push(UsePathSegment {
            separator: pending_separator.take(),
            spelling: std::mem::take(current),
        });
        Ok(())
    }
}
