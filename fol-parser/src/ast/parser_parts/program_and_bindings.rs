use super::*;

impl AstParser {
    pub fn new() -> Self {
        Self {
            routine_return_types: RefCell::new(HashMap::new()),
        }
    }

    /// Parse a token stream into an AST
    pub fn parse(
        &mut self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Vec<Box<dyn Glitch>>> {
        self.routine_return_types.borrow_mut().clear();
        self.seed_routine_return_types(tokens);

        let mut declarations = Vec::new();
        let mut errors: Vec<Box<dyn Glitch>> = Vec::new();

        for _ in 0..8_192 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(error) => {
                    errors.push(error);
                    break;
                }
            };

            let key = token.key();

            if key.is_eof() {
                break;
            }

            if key.is_illegal() {
                errors.push(Box::new(ParseError::from_token(
                    &token,
                    format!("Parser encountered illegal token '{}'", token.con()),
                )));
                if tokens.bump().is_none() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Var)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_var_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }

                if let Ok(current) = tokens.curr(false) {
                    let after = (
                        current.loc().row(),
                        current.loc().col(),
                        current.con().to_string(),
                    );
                    if before == after && tokens.bump().is_none() {
                        break;
                    }
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Let)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_let_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }

                if let Ok(current) = tokens.curr(false) {
                    let after = (
                        current.loc().row(),
                        current.loc().col(),
                        current.con().to_string(),
                    );
                    if before == after && tokens.bump().is_none() {
                        break;
                    }
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Con)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_con_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }

                if let Ok(current) = tokens.curr(false) {
                    let after = (
                        current.loc().row(),
                        current.loc().col(),
                        current.con().to_string(),
                    );
                    if before == after && tokens.bump().is_none() {
                        break;
                    }
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Use)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_use_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Def)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_def_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Ali)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_alias_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Typ)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_type_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Fun)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_fun_decl(tokens) {
                    Ok(node) => {
                        if let AstNode::FunDecl { body, .. } = &node {
                            declarations.extend(body.clone());
                        }
                        declarations.push(node);
                    }
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Log)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_log_decl(tokens) {
                    Ok(node) => {
                        if let AstNode::FunDecl { body, .. } = &node {
                            declarations.extend(body.clone());
                        }
                        declarations.push(node);
                    }
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Pro)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_pro_decl(tokens) {
                    Ok(node) => {
                        if let AstNode::ProDecl { body, .. } = &node {
                            declarations.extend(body.clone());
                        }
                        declarations.push(node);
                    }
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Return)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_return_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if key.is_ident() && self.lookahead_is_call(tokens) && self.can_start_assignment(tokens)
            {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_call_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Break)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_break_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Yeild)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_yield_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(
                key,
                KEYWORD::Keyword(BUILDIN::Panic)
                    | KEYWORD::Keyword(BUILDIN::Report)
                    | KEYWORD::Keyword(BUILDIN::Check)
                    | KEYWORD::Keyword(BUILDIN::Assert)
            ) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_builtin_call_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::When)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_when_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::If)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_if_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(
                key,
                KEYWORD::Keyword(BUILDIN::Loop)
                    | KEYWORD::Keyword(BUILDIN::For)
                    | KEYWORD::Keyword(BUILDIN::Each)
            ) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_loop_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if key.is_ident()
                && self.lookahead_is_assignment(tokens)
                && self.can_start_assignment(tokens)
            {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_assignment_stmt(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if key.is_ident() {
                declarations.push(AstNode::Identifier {
                    name: token.con().trim().to_string(),
                });
                if tokens.bump().is_none() {
                    break;
                }
                continue;
            }

            if key.is_literal() {
                match self.parse_lexer_literal(&token) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        if errors.is_empty() {
            Ok(AstNode::Program { declarations })
        } else {
            Err(errors)
        }
    }

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
                Ok(name) if name.key().is_ident() => {
                    let parsed = name.con().trim().to_string();
                    let _ = tokens.bump();
                    parsed
                }
                _ => {
                    if tokens.bump().is_none() {
                        break;
                    }
                    continue;
                }
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

    pub(super) fn parse_lexer_literal(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let raw = token.con().trim();

        match token.key() {
            fol_lexer::token::KEYWORD::Literal(LITERAL::Stringy) => {
                Ok(AstNode::Literal(Literal::String(raw.to_string())))
            }
            fol_lexer::token::KEYWORD::Literal(LITERAL::Bool) => match raw {
                "true" => Ok(AstNode::Literal(Literal::Boolean(true))),
                "false" => Ok(AstNode::Literal(Literal::Boolean(false))),
                _ => Ok(AstNode::Identifier {
                    name: raw.to_string(),
                }),
            },
            _ => self.parse_literal(raw),
        }
    }

    pub(super) fn parse_var_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.parse_binding_decl(tokens, "var", vec![VarOption::Mutable, VarOption::Normal])
    }

    pub(super) fn parse_let_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.parse_binding_decl(tokens, "let", vec![VarOption::Immutable, VarOption::Normal])
    }

    pub(super) fn parse_con_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.parse_binding_decl(tokens, "con", vec![VarOption::Immutable, VarOption::Normal])
    }

    pub(super) fn parse_binding_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        keyword: &str,
        default_options: Vec<VarOption>,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut type_hint = None;
        let mut value = None;

        if tokens.bump().is_none() {
            return Err(Box::new(ParseError {
                message: format!("Unexpected EOF after '{}' declaration", keyword),
                file: None,
                line: 1,
                column: 1,
                length: 1,
            }));
        }
        self.skip_ignorable(tokens);
        let options = self.parse_binding_options(tokens, default_options)?;
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = if name_token.key().is_ident() {
            let parsed_name = name_token.con().trim().to_string();
            let _ = tokens.bump();
            parsed_name
        } else {
            return Err(Box::new(ParseError::from_token(
                &name_token,
                format!("Expected identifier after '{}'", keyword),
            )));
        };

        self.skip_ignorable(tokens);

        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                type_hint = Some(self.parse_type_reference_tokens(tokens)?);
            }
        }

        self.skip_ignorable(tokens);

        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                let parsed_value = self.parse_logical_expression(tokens)?;
                value = Some(Box::new(parsed_value));
            }
        }

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::VarDecl {
            options,
            name,
            type_hint,
            value,
        })
    }

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

    pub(super) fn parse_use_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
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

        let name_token = tokens.curr(false)?;
        if !name_token.key().is_ident() {
            return Err(Box::new(ParseError::from_token(
                &name_token,
                "Expected use declaration name".to_string(),
            )));
        }
        let name = name_token.con().trim().to_string();
        let _ = tokens.bump();

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

        self.skip_ignorable(tokens);
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start use path".to_string(),
            )));
        }
        let _ = tokens.bump();

        let path = self.parse_use_path(tokens)?;

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::UseDecl {
            options,
            name,
            path_type,
            path,
        })
    }

    pub(super) fn parse_use_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<UseOption>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(Vec::new()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(Vec::new());
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let token = tokens.curr(false)?;
        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            let _ = tokens.bump();
            return Ok(Vec::new());
        }

        Err(Box::new(ParseError::from_token(
            &token,
            "Unknown use option".to_string(),
        )))
    }
}
