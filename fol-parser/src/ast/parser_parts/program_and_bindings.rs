use super::*;
use crate::ast::BindingPattern;

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

            if self.lookahead_binding_alternative(tokens).is_some() {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_binding_alternative_decl(tokens) {
                    Ok(nodes) => declarations.extend(nodes),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
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
                    Ok(nodes) => declarations.extend(nodes),
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
                    Ok(nodes) => declarations.extend(nodes),
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
                    Ok(nodes) => declarations.extend(nodes),
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

            if matches!(key, KEYWORD::Keyword(BUILDIN::Lab)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_lab_decl(tokens) {
                    Ok(nodes) => declarations.extend(nodes),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
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
                    Ok(nodes) => declarations.extend(nodes),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Seg)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_seg_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Imp)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_imp_decl(tokens) {
                    Ok(node) => declarations.push(node),
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Std)) && self.lookahead_is_std_decl(tokens) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_std_decl(tokens) {
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
                    Ok(nodes) => declarations.extend(nodes),
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

            if (AstParser::token_can_be_logical_name(&key)
                || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
                && self.lookahead_is_call(tokens)
                && self.can_start_assignment(tokens)
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

            if matches!(key, KEYWORD::Symbol(SYMBOL::Dot))
                && self.lookahead_is_dot_builtin_call(tokens)
                && self.can_start_assignment(tokens)
            {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_dot_builtin_call_expr(tokens) {
                    Ok(node) => {
                        declarations.push(node);
                        self.consume_optional_semicolon(tokens);
                    }
                    Err(error) => errors.push(error),
                }
                self.bump_if_no_progress(tokens, before);
                if tokens.curr(false).is_err() {
                    break;
                }
                continue;
            }

            if (matches!(key, KEYWORD::Symbol(SYMBOL::RoundO) | KEYWORD::Symbol(SYMBOL::Dot))
                || AstParser::token_can_be_logical_name(&key)
                || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
                && self.lookahead_is_general_invoke(tokens, matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)))
                && self.can_start_assignment(tokens)
            {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_invoke_stmt(tokens) {
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

            if matches!(key, KEYWORD::Keyword(BUILDIN::Select)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                match self.parse_select_stmt(tokens) {
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
                KEYWORD::Keyword(BUILDIN::While)
                    | KEYWORD::Keyword(BUILDIN::Loop)
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

            if (AstParser::token_can_be_logical_name(&key)
                || matches!(key, KEYWORD::Literal(LITERAL::Stringy)))
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

            if AstParser::token_can_be_logical_name(&key) {
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

    pub(super) fn parse_lexer_literal(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let raw = token.con().trim();

        match token.key() {
            fol_lexer::token::KEYWORD::Literal(LITERAL::Stringy) => self.parse_literal(raw),
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
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        self.parse_binding_decl(tokens, "var", vec![VarOption::Mutable, VarOption::Normal])
    }

    pub(super) fn parse_let_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        self.parse_binding_decl(tokens, "let", vec![VarOption::Immutable, VarOption::Normal])
    }

    pub(super) fn parse_con_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        self.parse_binding_decl(tokens, "con", vec![VarOption::Immutable, VarOption::Normal])
    }

    pub(super) fn parse_lab_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let nodes =
            self.parse_binding_decl(tokens, "lab", vec![VarOption::Immutable, VarOption::Normal])?;
        Ok(nodes
            .into_iter()
            .map(|node| match node {
                AstNode::VarDecl {
                    options,
                    name,
                    type_hint,
                    value,
                } => AstNode::LabDecl {
                    options,
                    name,
                    type_hint,
                    value,
                },
                AstNode::DestructureDecl {
                    options,
                    pattern,
                    type_hint,
                    value,
                    ..
                } => AstNode::DestructureDecl {
                    options,
                    is_label: true,
                    pattern,
                    type_hint,
                    value,
                },
                other => other,
            })
            .collect())
    }

    pub(super) fn parse_binding_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        keyword: &str,
        default_options: Vec<VarOption>,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
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

        if let Ok(open_group) = tokens.curr(false) {
            if matches!(open_group.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
                return self.parse_binding_group(tokens, options);
            }
        }

        let mut nodes = Vec::new();
        for _ in 0..256 {
            let patterns = self.parse_binding_pattern_list(tokens, keyword)?;
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
                    values = self.parse_binding_values(tokens, !is_destructuring && patterns.len() == 1)?;
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
                            message: format!("Unsupported plain binding pattern: {other:?}"),
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
            let next = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Semi)) {
                let _ = tokens.bump();
                break;
            }
            if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma))
                && self.lookahead_starts_binding_segment(tokens)
            {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                continue;
            }

            break;
        }

        self.consume_optional_semicolon(tokens);
        Ok(nodes)
    }

    pub(super) fn parse_binding_pattern_list(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        keyword: &str,
    ) -> Result<Vec<BindingPattern>, Box<dyn Glitch>> {
        let mut patterns = Vec::new();

        for _ in 0..256 {
            patterns.push(self.parse_binding_pattern(tokens, keyword)?);
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

        Ok(patterns)
    }

    pub(super) fn parse_binding_pattern(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        keyword: &str,
    ) -> Result<BindingPattern, Box<dyn Glitch>> {
        let token = tokens.curr(false)?;

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            let _ = tokens.bump();
            let mut parts = Vec::new();
            for _ in 0..256 {
                self.skip_ignorable(tokens);
                let current = tokens.curr(false)?;
                if matches!(current.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                    let _ = tokens.bump();
                    return Ok(BindingPattern::Sequence(parts));
                }

                parts.push(self.parse_binding_pattern(tokens, keyword)?);
                self.skip_ignorable(tokens);

                let separator = tokens.curr(false)?;
                if matches!(separator.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                    let _ = tokens.bump();
                    continue;
                }
                if matches!(separator.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                    let _ = tokens.bump();
                    return Ok(BindingPattern::Sequence(parts));
                }

                return Err(Box::new(ParseError::from_token(
                    &separator,
                    "Expected ',' or ')' in binding pattern".to_string(),
                )));
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Binding pattern exceeded parser limit".to_string(),
            )));
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Star)) {
            let star = token.clone();
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
            let name_token = tokens.curr(false)?;
            let name = if matches!(name_token.key(), KEYWORD::Symbol(SYMBOL::Under)) {
                "_".to_string()
            } else {
                Self::token_to_named_label(&name_token).ok_or_else(|| {
                    Box::new(ParseError::from_token(
                        &name_token,
                        "Expected binding name after '*'".to_string(),
                    )) as Box<dyn Glitch>
                })?
            };
            let _ = tokens.bump();
            if matches!(name_token.key(), KEYWORD::Symbol(SYMBOL::Star)) {
                return Err(Box::new(ParseError::from_token(
                    &star,
                    "Nested '*' binding patterns are not allowed".to_string(),
                )));
            }
            return Ok(BindingPattern::Rest(name));
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Under)) {
            let _ = tokens.bump();
            return Ok(BindingPattern::Name("_".to_string()));
        }

        let name = Self::token_to_named_label(&token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &token,
                format!("Expected identifier after '{}'", keyword),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();
        Ok(BindingPattern::Name(name))
    }

    pub(super) fn build_binding_nodes(
        &self,
        options: Vec<VarOption>,
        names: Vec<String>,
        type_hint: Option<FolType>,
        values: Vec<AstNode>,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let assigned_values = match values.len() {
            0 => vec![None; names.len()],
            1 => vec![Some(values[0].clone()); names.len()],
            n if n == names.len() => values.into_iter().map(Some).collect(),
            _ => return Err(Box::new(ParseError {
                message:
                    "Binding value count must match declared names or provide a single shared value"
                        .to_string(),
                file: None,
                line: 0,
                column: 0,
                length: 0,
            })),
        };

        Ok(names
            .into_iter()
            .zip(assigned_values)
            .map(|(name, value)| AstNode::VarDecl {
                options: options.clone(),
                name,
                type_hint: type_hint.clone(),
                value: value.map(Box::new),
            })
            .collect())
    }

    pub(super) fn build_destructure_binding_node(
        &self,
        options: Vec<VarOption>,
        patterns: Vec<BindingPattern>,
        type_hint: Option<FolType>,
        values: Vec<AstNode>,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        if values.len() != 1 {
            return Err(Box::new(ParseError {
                message: "Destructuring bindings require exactly one source value".to_string(),
                file: None,
                line: 0,
                column: 0,
                length: 0,
            }));
        }

        Ok(AstNode::DestructureDecl {
            options,
            is_label: false,
            pattern: BindingPattern::Sequence(patterns),
            type_hint,
            value: Box::new(values.into_iter().next().expect("single destructure value")),
        })
    }

}
