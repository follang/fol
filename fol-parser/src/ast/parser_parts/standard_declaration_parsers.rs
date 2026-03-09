use super::*;

impl AstParser {
    pub(super) fn lookahead_is_std_decl(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        if !matches!(
            tokens.curr(false).map(|token| token.key().clone()),
            Ok(KEYWORD::Keyword(BUILDIN::Std))
        ) {
            return false;
        }

        let mut significant = tokens.next_vec().into_iter().filter_map(Result::ok).filter(|token| {
            let key = token.key();
            !key.is_void() && !key.is_comment()
        });

        let Some(mut name_token) = significant.next() else {
            return false;
        };
        if matches!(name_token.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            let mut depth = 1usize;
            while depth > 0 {
                let Some(token) = significant.next() else {
                    return false;
                };
                match token.key() {
                    KEYWORD::Symbol(SYMBOL::SquarO) => depth += 1,
                    KEYWORD::Symbol(SYMBOL::SquarC) => depth -= 1,
                    _ => {}
                }
            }
            let Some(next_name) = significant.next() else {
                return false;
            };
            name_token = next_name;
        }
        if Self::token_to_named_label(&name_token).is_none() {
            return false;
        }

        matches!(
            significant.next().map(|token| token.key().clone()),
            Some(KEYWORD::Symbol(SYMBOL::Colon))
        )
    }

    pub(super) fn parse_std_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let std_token = tokens.curr(false)?;
        if !matches!(std_token.key(), KEYWORD::Keyword(BUILDIN::Std)) {
            return Err(Box::new(ParseError::from_token(
                &std_token,
                "Expected 'std' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let options = self.parse_decl_visibility_options(tokens, "standard")?;
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &name_token,
                "Expected standard name after 'std'".to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' after standard name".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let kind_token = tokens.curr(false)?;
        let kind = match kind_token.con().trim() {
            "pro" => StandardKind::Protocol,
            "blu" => StandardKind::Blueprint,
            "ext" => StandardKind::Extended,
            other => {
                return Err(Box::new(ParseError::from_token(
                    &kind_token,
                    format!("Unknown standard kind '{}'", other),
                )))
            }
        };
        let _ = tokens.bump();
        let kind_options = match kind {
            StandardKind::Protocol => {
                self.parse_decl_visibility_options(tokens, "protocol standard kind")?
            }
            StandardKind::Blueprint => {
                self.parse_decl_visibility_options(tokens, "blueprint standard kind")?
            }
            StandardKind::Extended => {
                self.parse_decl_visibility_options(tokens, "extended standard kind")?
            }
        };

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' before standard body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start standard body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let body = match kind {
            StandardKind::Protocol => self.parse_standard_protocol_body(tokens)?,
            StandardKind::Blueprint => self.parse_standard_blueprint_body(tokens)?,
            StandardKind::Extended => self.parse_standard_extended_body(tokens)?,
        };
        self.consume_optional_semicolon(tokens);

        Ok(AstNode::StdDecl {
            options,
            name,
            kind,
            kind_options,
            body,
        })
    }

    fn parse_standard_protocol_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut seen_members = HashSet::new();
        let mut anchor_token = None;

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if token.key().is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    "Expected '}' to close standard body".to_string(),
                )));
            }

            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Log)
                    | KEYWORD::Keyword(BUILDIN::Pro)
            ) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_standard_routine_signature(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Ali)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_alias_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Typ)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_type_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Con)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_con_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Protocol standards currently support only routine, alias, type, and constant declarations"
                    .to_string(),
            )));
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            "Standard body exceeded parser limit".to_string(),
        )))
    }

    fn parse_standard_blueprint_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut seen_members = HashSet::new();
        let mut anchor_token = None;

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if token.key().is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    "Expected '}' to close standard body".to_string(),
                )));
            }

            if self.lookahead_binding_alternative(tokens).is_some() {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_binding_alternative_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Var)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_var_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Lab)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_lab_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Con)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_con_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Log)
                    | KEYWORD::Keyword(BUILDIN::Pro)
            ) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_standard_routine_signature(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Ali)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_alias_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Typ)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_type_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Typ)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_type_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Typ)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_type_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Typ)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_type_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if self.lookahead_binding_alternative(tokens).is_some() {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_binding_alternative_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Blueprint standards currently support only field, routine, alias, and type declarations"
                    .to_string(),
            )));
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            "Standard body exceeded parser limit".to_string(),
        )))
    }

    fn parse_standard_extended_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut seen_members = HashSet::new();
        let mut anchor_token = None;

        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if token.key().is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    "Expected '}' to close standard body".to_string(),
                )));
            }

            if matches!(
                token.key(),
                KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Log)
                    | KEYWORD::Keyword(BUILDIN::Pro)
            ) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_standard_routine_signature(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Var)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_var_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Lab)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_lab_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Con)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_con_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Ali)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_alias_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if matches!(token.key(), KEYWORD::Keyword(BUILDIN::Typ)) {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let member = self.parse_type_decl(tokens)?;
                let key = self.standard_member_key(&member);
                if !seen_members.insert(key.clone()) {
                    return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                }
                body.push(member);
                continue;
            }

            if self.lookahead_binding_alternative(tokens).is_some() {
                let member_anchor = self.peek_standard_member_anchor_token(tokens);
                let members = self.parse_binding_alternative_decl(tokens)?;
                for member in members {
                    let key = self.standard_member_key(&member);
                    if !seen_members.insert(key.clone()) {
                        return Err(self.duplicate_standard_member_error(member_anchor, &token, &key));
                    }
                    body.push(member);
                }
                continue;
            }

            return Err(Box::new(ParseError::from_token(
                &token,
                "Extended standards currently support only routine, field, alias, and type declarations"
                    .to_string(),
            )));
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            "Standard body exceeded parser limit".to_string(),
        )))
    }

    pub(super) fn parse_standard_routine_signature(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let routine_token = tokens.curr(false)?;
        let routine_kind = routine_token.key().clone();
        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let options = self.parse_routine_options(tokens)?;
        self.skip_ignorable(tokens);

        let (_, name) = self.parse_routine_name_with_optional_receiver(
            tokens,
            "Expected routine name in standard signature",
        )?;
        let (generics, params) =
            self.parse_routine_generics_and_params(tokens, "Expected '(' after routine name")?;
        self.skip_ignorable(tokens);
        let captures = self.parse_optional_routine_capture_list(tokens)?;
        self.ensure_unique_capture_names(&captures)?;

        self.skip_ignorable(tokens);
        let mut return_type = None;
        let mut error_type = None;
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                return_type = Some(self.parse_type_reference_tokens(tokens)?);

                self.skip_ignorable(tokens);
                if let Ok(err_sep) = tokens.curr(false) {
                    if matches!(err_sep.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                        let _ = tokens.bump();
                        self.skip_ignorable(tokens);
                        error_type = Some(self.parse_type_reference_tokens(tokens)?);
                    }
                }
            }
        }

        self.skip_ignorable(tokens);
        let mut body = Vec::new();
        let mut inquiries = Vec::new();
        let next = tokens.curr(false)?;
        match next.key() {
            KEYWORD::Symbol(SYMBOL::Semi) => {
                let _ = tokens.bump();
            }
            KEYWORD::Symbol(SYMBOL::Equal) | KEYWORD::Operator(OPERATOR::Flow) => {
                if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                }
                let (parsed_body, parsed_inquiries) = self.parse_named_routine_body(
                    tokens,
                    "Expected '{' or '=>' to start standard routine body",
                    "Expected '}' to close standard routine body",
                )?;
                body = parsed_body;
                inquiries = parsed_inquiries;
            }
            _ => {
                return Err(Box::new(ParseError::from_token(
                    &next,
                    "Expected ';', '=', or '=>' after standard routine declaration".to_string(),
                )))
            }
        }

        Ok(match routine_kind {
            KEYWORD::Keyword(BUILDIN::Pro) => AstNode::ProDecl {
                options,
                generics,
                name,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            },
            _ => AstNode::FunDecl {
                options,
                generics,
                name,
                captures,
                params,
                return_type,
                error_type,
                body,
                inquiries,
            },
        })
    }

    fn standard_member_key(&self, node: &AstNode) -> String {
        match node {
            AstNode::FunDecl { name, params, .. } | AstNode::ProDecl { name, params, .. } => {
                format!("{}#{}", name, params.len())
            }
            AstNode::AliasDecl { name, .. } => name.clone(),
            AstNode::TypeDecl { name, .. } => name.clone(),
            AstNode::VarDecl { name, .. } | AstNode::LabDecl { name, .. } => name.clone(),
            _ => String::new(),
        }
    }

    fn duplicate_standard_member_error(
        &self,
        anchor: Option<fol_lexer::lexer::stage3::element::Element>,
        fallback: &fol_lexer::lexer::stage3::element::Element,
        key: &str,
    ) -> Box<dyn Glitch> {
        let token = anchor.unwrap_or_else(|| fallback.clone());
        Box::new(ParseError::from_token(
            &token,
            format!("Duplicate standard member '{}'", key),
        ))
    }

    fn peek_standard_member_anchor_token(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> Option<fol_lexer::lexer::stage3::element::Element> {
        let mut significant = vec![tokens.curr(false).ok()?];
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };
            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }
            significant.push(token);
            if significant.len() >= 32 {
                break;
            }
        }

        let mut index = 0;
        if matches!(
            significant.get(index)?.key(),
            KEYWORD::Symbol(SYMBOL::Plus)
                | KEYWORD::Symbol(SYMBOL::Minus)
                | KEYWORD::Symbol(SYMBOL::Home)
                | KEYWORD::Symbol(SYMBOL::Bang)
                | KEYWORD::Symbol(SYMBOL::Query)
                | KEYWORD::Symbol(SYMBOL::At)
        ) {
            index += 1;
        }

        match significant.get(index)?.key() {
            KEYWORD::Keyword(BUILDIN::Var)
            | KEYWORD::Keyword(BUILDIN::Lab)
            | KEYWORD::Keyword(BUILDIN::Con) => {
                index += 1;
                index = self.skip_balanced_window(
                    &significant,
                    index,
                    KEYWORD::Symbol(SYMBOL::SquarO),
                    KEYWORD::Symbol(SYMBOL::SquarC),
                )?;
                self.find_named_label_in_window(&significant, index)
            }
            KEYWORD::Keyword(BUILDIN::Fun)
            | KEYWORD::Keyword(BUILDIN::Log)
            | KEYWORD::Keyword(BUILDIN::Pro) => {
                index += 1;
                index = self.skip_balanced_window(
                    &significant,
                    index,
                    KEYWORD::Symbol(SYMBOL::SquarO),
                    KEYWORD::Symbol(SYMBOL::SquarC),
                )?;
                index = self.skip_balanced_window(
                    &significant,
                    index,
                    KEYWORD::Symbol(SYMBOL::RoundO),
                    KEYWORD::Symbol(SYMBOL::RoundC),
                )?;
                self.find_named_label_in_window(&significant, index)
            }
            KEYWORD::Keyword(BUILDIN::Ali) => {
                index += 1;
                self.find_named_label_in_window(&significant, index)
            }
            KEYWORD::Keyword(BUILDIN::Typ) => {
                index += 1;
                index = self.skip_balanced_window(
                    &significant,
                    index,
                    KEYWORD::Symbol(SYMBOL::RoundO),
                    KEYWORD::Symbol(SYMBOL::RoundC),
                )?;
                self.find_named_label_in_window(&significant, index)
            }
            _ => None,
        }
    }

    fn skip_balanced_window(
        &self,
        tokens: &[fol_lexer::lexer::stage3::element::Element],
        start: usize,
        open: KEYWORD,
        close: KEYWORD,
    ) -> Option<usize> {
        if tokens.get(start)?.key() != open.clone() {
            return Some(start);
        }

        let mut depth = 0usize;
        for (index, token) in tokens.iter().enumerate().skip(start) {
            if token.key() == open {
                depth += 1;
            } else if token.key() == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index + 1);
                }
            }
        }

        None
    }

    fn find_named_label_in_window(
        &self,
        tokens: &[fol_lexer::lexer::stage3::element::Element],
        start: usize,
    ) -> Option<fol_lexer::lexer::stage3::element::Element> {
        tokens
            .iter()
            .skip(start)
            .find(|token| Self::token_to_named_label(token).is_some())
            .cloned()
    }
}
