use super::*;

impl AstParser {
    pub(super) fn seed_routine_return_types(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) {
        let source_path = Self::extract_source_path(tokens);

        let Some(path) = source_path else {
            return;
        };

        let mut file_stream = match fol_stream::FileStream::from_file(&path) {
            Ok(stream) => stream,
            Err(_) => return,
        };

        let mut scan_tokens = fol_lexer::lexer::stage3::Elements::init(&mut file_stream);
        let signatures = self.collect_routine_signatures(&mut scan_tokens);
        self.routine_return_types.borrow_mut().extend(signatures);
    }

    pub(super) fn extract_source_path(
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> Option<String> {
        if let Ok(token) = tokens.curr(false) {
            if let Some(path) = token.loc().source().map(|src| src.path(true)) {
                return Some(path);
            }
        }

        if let Ok(token) = tokens.curr(true) {
            if let Some(path) = token.loc().source().map(|src| src.path(true)) {
                return Some(path);
            }
        }

        for index in 0..16 {
            if let Ok(token) = tokens.peek(index, false) {
                if let Some(path) = token.loc().source().map(|src| src.path(true)) {
                    return Some(path);
                }
            }

            if let Ok(token) = tokens.peek(index, true) {
                if let Some(path) = token.loc().source().map(|src| src.path(true)) {
                    return Some(path);
                }
            }
        }

        None
    }

    pub(super) fn collect_routine_signatures(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> HashMap<String, FolType> {
        let mut signatures = HashMap::new();

        for _ in 0..16_384 {
            self.skip_ignorable(tokens);
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if token.key().is_eof() {
                break;
            }

            if !matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Log)
                    | KEYWORD::Keyword(BUILDIN::Pro)
            ) {
                if tokens.bump().is_none() {
                    break;
                }
                continue;
            }

            match self.previous_significant_key(tokens) {
                None
                | Some(KEYWORD::Symbol(SYMBOL::CurlyO))
                | Some(KEYWORD::Symbol(SYMBOL::CurlyC))
                | Some(KEYWORD::Symbol(SYMBOL::Semi))
                | Some(KEYWORD::Void(VOID::EndLine)) => {}
                _ => {
                    if tokens.bump().is_none() {
                        break;
                    }
                    continue;
                }
            }

            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let mut receiver_name: Option<String> = None;
            if let Ok(open) = tokens.curr(false) {
                if matches!(open.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);

                    if let Ok(FolType::Named { name }) = self.parse_type_reference_tokens(tokens) {
                        receiver_name = Some(name);
                    }

                    self.skip_ignorable(tokens);
                    if let Ok(close) = tokens.curr(false) {
                        if matches!(close.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                            let _ = tokens.bump();
                        }
                    }
                    self.skip_ignorable(tokens);
                }
            }

            let routine_name = match tokens.curr(false) {
                Ok(name) => match Self::token_to_named_label(&name) {
                    Some(parsed) => {
                        let _ = tokens.bump();
                        parsed
                    }
                    None => {
                        if tokens.bump().is_none() {
                            break;
                        }
                        continue;
                    }
                },
                Err(_) => break,
            };

            self.skip_ignorable(tokens);
            let mut param_arity = 0usize;
            if let Ok(open_params) = tokens.curr(false) {
                if matches!(open_params.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                    param_arity = self.scan_parameter_arity(tokens);
                }
            }

            self.skip_ignorable(tokens);
            let mut return_type = None;
            if let Ok(colon) = tokens.curr(false) {
                if matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    return_type = self.parse_type_reference_tokens(tokens).ok();
                }
            }

            if let Some(rt) = return_type {
                signatures
                    .entry(Self::callable_key(&routine_name, param_arity))
                    .or_insert(rt.clone());
                if let Some(receiver) = receiver_name {
                    signatures
                        .entry(Self::callable_key(
                            &format!("{}.{}", receiver, routine_name),
                            param_arity,
                        ))
                        .or_insert(rt);
                }
            }
        }

        signatures
    }

    pub(super) fn scan_parameter_arity(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> usize {
        let mut depth: usize = 0;
        let mut count: usize = 0;
        let mut saw_token_in_slot = false;

        for _ in 0..8_192 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return count,
            };

            if token.key().is_space() {
                if tokens.bump().is_none() {
                    return count;
                }
                continue;
            }

            match token.key() {
                KEYWORD::Symbol(SYMBOL::RoundO) => {
                    depth += 1;
                    if depth > 1 {
                        saw_token_in_slot = true;
                    }
                }
                KEYWORD::Symbol(SYMBOL::RoundC) => {
                    if depth == 1 {
                        if saw_token_in_slot {
                            count += 1;
                        }
                        let _ = tokens.bump();
                        return count;
                    }
                    if depth == 0 {
                        return count;
                    }
                    depth -= 1;
                }
                KEYWORD::Symbol(SYMBOL::Comma) => {
                    if depth == 1 {
                        if saw_token_in_slot {
                            count += 1;
                        }
                        saw_token_in_slot = false;
                    } else {
                        saw_token_in_slot = true;
                    }
                }
                _ => {
                    if depth >= 1 {
                        saw_token_in_slot = true;
                    }
                }
            }

            if tokens.bump().is_none() {
                return count;
            }
        }

        count
    }

}
