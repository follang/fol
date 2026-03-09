use super::*;

impl AstParser {
    pub(super) fn parse_routine_header_list(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<
        (
            Vec<Parameter>,
            Option<fol_lexer::lexer::stage3::element::Element>,
        ),
        Box<dyn Glitch>,
    > {
        let mut params = Vec::new();
        let mut first_untyped = None;

        for _ in 0..128 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok((params, first_untyped));
            }

            let first_name = Self::token_to_named_label(&token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &token,
                    "Expected generic parameter name".to_string(),
                )) as Box<dyn Glitch>
            })?;

            let mut names = vec![first_name];
            let _ = tokens.bump();

            self.skip_ignorable(tokens);

            loop {
                let next = tokens.curr(false)?;
                if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    break;
                }
                if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    let name_token = tokens.curr(false)?;
                    let grouped_name = Self::token_to_named_label(&name_token).ok_or_else(|| {
                        Box::new(ParseError::from_token(
                            &name_token,
                            "Expected parameter name after ','".to_string(),
                        )) as Box<dyn Glitch>
                    })?;
                    names.push(grouped_name);
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    continue;
                }
                break;
            }

            let mut is_variadic = false;
            let param_type = if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    if let Ok(token) = tokens.curr(false) {
                        if matches!(token.key(), KEYWORD::Operator(OPERATOR::Dotdotdot))
                            || token.con().trim() == "..."
                        {
                            is_variadic = true;
                            let _ = tokens.bump();
                            self.skip_ignorable(tokens);
                        }
                    }
                    let base_type = self.parse_type_reference_tokens(tokens)?;
                    if is_variadic {
                        FolType::Sequence {
                            element_type: Box::new(base_type),
                        }
                    } else {
                        base_type
                    }
                } else {
                    if first_untyped.is_none() {
                        first_untyped = Some(token.clone());
                    }
                    FolType::Named {
                        name: "any".to_string(),
                    }
                }
            } else {
                FolType::Named {
                    name: "any".to_string(),
                }
            };

            self.skip_ignorable(tokens);
            let default = if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);

                    let next = tokens.curr(false)?;
                    if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma))
                        || matches!(next.key(), KEYWORD::Symbol(SYMBOL::RoundC))
                        || next.key().is_terminal()
                    {
                        return Err(Box::new(ParseError::from_token(
                            &next,
                            "Expected default value expression after '=' in parameter".to_string(),
                        )));
                    }

                    Some(self.parse_logical_expression(tokens)?)
                } else {
                    None
                }
            } else {
                None
            };

            if is_variadic && default.is_some() {
                let token = tokens.curr(false)?;
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Variadic parameters cannot have default values".to_string(),
                )));
            }

            for name in names {
                params.push(Parameter {
                    name: name.clone(),
                    param_type: param_type.clone(),
                    is_borrowable: name.chars().all(|ch| {
                        !ch.is_ascii_lowercase() && (ch.is_ascii_alphanumeric() || ch == '_')
                    }),
                    default: default.clone(),
                });
            }

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma))
                || matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Semi))
            {
                if is_variadic {
                    return Err(Box::new(ParseError::from_token(
                        &sep,
                        "Variadic parameter must be the last parameter".to_string(),
                    )));
                }
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok((params, first_untyped));
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ')' after generic parameter".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Generic parsing exceeded safety bound".to_string(),
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }))
    }

    pub(super) fn parameters_to_generics(
        &self,
        params: Vec<Parameter>,
    ) -> Result<Vec<Generic>, Box<dyn Glitch>> {
        params
            .into_iter()
            .map(|param| {
                if param.default.is_some() {
                    return Err(Box::new(ParseError {
                        message: "Default values are not allowed in routine generic headers"
                            .to_string(),
                        file: None,
                        line: 1,
                        column: 1,
                        length: 1,
                    }) as Box<dyn Glitch>);
                }

                if matches!(param.param_type, FolType::Sequence { .. }) {
                    return Err(Box::new(ParseError {
                        message: "Variadic parameters are not allowed in routine generic headers"
                            .to_string(),
                        file: None,
                        line: 1,
                        column: 1,
                        length: 1,
                    }) as Box<dyn Glitch>);
                }

                let constraints = match param.param_type {
                    FolType::Named { name } if name == "any" => Vec::new(),
                    other => vec![other],
                };

                Ok(Generic {
                    name: param.name,
                    constraints,
                })
            })
            .collect()
    }

    pub(super) fn ensure_unique_parameter_names(
        &self,
        params: &[Parameter],
        kind: &str,
    ) -> Result<(), Box<dyn Glitch>> {
        let mut seen_names = HashSet::new();
        for param in params {
            if !seen_names.insert(param.name.clone()) {
                return Err(Box::new(ParseError {
                    message: format!("Duplicate {} name '{}'", kind, param.name),
                    file: None,
                    line: 1,
                    column: 1,
                    length: 1,
                }));
            }
        }
        Ok(())
    }

    pub(super) fn parse_generic_list(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<Generic>, Box<dyn Glitch>> {
        let mut generics = Vec::new();
        let mut seen_names = HashSet::new();

        for _ in 0..128 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok(generics);
            }

            let name = Self::token_to_named_label(&token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &token,
                    "Expected generic parameter name".to_string(),
                )) as Box<dyn Glitch>
            })?;
            if !seen_names.insert(name.clone()) {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    format!("Duplicate generic name '{}'", name),
                )));
            }
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let mut constraints = Vec::new();
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                    constraints.push(self.parse_type_reference_tokens(tokens)?);
                }
            }

            generics.push(Generic { name, constraints });

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma))
                || matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Semi))
            {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok(generics);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ')' after generic parameter".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Generic parsing exceeded safety bound".to_string(),
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }))
    }

    pub(super) fn parse_routine_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<FunOption>, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(vec![FunOption::Mutable]),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(vec![FunOption::Mutable]);
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
                "+" | "exp" | "export" => FunOption::Export,
                "mut" | "mutable" => FunOption::Mutable,
                "itr" | "iter" | "iterator" => FunOption::Iterator,
                _ => {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Unknown routine option".to_string(),
                    )))
                }
            };
            options.push(option);
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(
                sep.key(),
                KEYWORD::Symbol(SYMBOL::Comma) | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                let _ = tokens.bump();
                self.skip_ignorable(tokens);
                if matches!(
                    tokens.curr(false).map(|token| token.key()),
                    Ok(KEYWORD::Symbol(SYMBOL::SquarC))
                ) {
                    let _ = tokens.bump();
                    return Ok(options);
                }
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(options);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ']' in routine options".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Routine options exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }

    pub(super) fn parse_parameter_list(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<Parameter>, Box<dyn Glitch>> {
        let mut params = Vec::new();
        let mut seen_names = HashSet::new();

        for _ in 0..512 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok(params);
            }

            let first_name = Self::token_to_named_label(&token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &token,
                    "Expected parameter name".to_string(),
                )) as Box<dyn Glitch>
            })?;
            if !seen_names.insert(first_name.clone()) {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    format!("Duplicate parameter name '{}'", first_name),
                )));
            }
            let mut names = vec![first_name];
            let _ = tokens.bump();

            loop {
                self.skip_ignorable(tokens);
                let next = tokens.curr(false)?;
                if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
                    break;
                }
                if !matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                    return Err(Box::new(ParseError::from_token(
                        &next,
                        "Expected ':' after parameter name".to_string(),
                    )));
                }
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                let name_token = tokens.curr(false)?;
                let grouped_name = Self::token_to_named_label(&name_token).ok_or_else(|| {
                    Box::new(ParseError::from_token(
                        &name_token,
                        "Expected parameter name after ','".to_string(),
                    )) as Box<dyn Glitch>
                })?;
                if !seen_names.insert(grouped_name.clone()) {
                    return Err(Box::new(ParseError::from_token(
                        &name_token,
                        format!("Duplicate parameter name '{}'", grouped_name),
                    )));
                }
                names.push(grouped_name);
                let _ = tokens.bump();
            }

            let _colon = tokens.curr(false)?;
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let mut is_variadic = false;
            if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Operator(OPERATOR::Dotdotdot))
                    || token.con().trim() == "..."
                {
                    is_variadic = true;
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);
                }
            }

            let base_type = self.parse_type_reference_tokens(tokens)?;
            let param_type = if is_variadic {
                FolType::Sequence {
                    element_type: Box::new(base_type),
                }
            } else {
                base_type
            };
            self.skip_ignorable(tokens);

            let default = if let Ok(token) = tokens.curr(false) {
                if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);

                    let next = tokens.curr(false)?;
                    if matches!(next.key(), KEYWORD::Symbol(SYMBOL::Comma))
                        || matches!(next.key(), KEYWORD::Symbol(SYMBOL::Semi))
                        || matches!(next.key(), KEYWORD::Symbol(SYMBOL::RoundC))
                        || next.key().is_terminal()
                    {
                        return Err(Box::new(ParseError::from_token(
                            &next,
                            "Expected default value expression after '=' in parameter".to_string(),
                        )));
                    }

                    Some(self.parse_logical_expression(tokens)?)
                } else {
                    None
                }
            } else {
                None
            };

            if is_variadic && default.is_some() {
                let token = tokens.curr(false)?;
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Variadic parameters cannot have default values".to_string(),
                )));
            }

            for param_name in names {
                params.push(Parameter {
                    name: param_name.clone(),
                    param_type: param_type.clone(),
                    is_borrowable: param_name.chars().all(|ch| {
                        !ch.is_ascii_lowercase() && (ch.is_ascii_alphanumeric() || ch == '_')
                    }),
                    default: default.clone(),
                });
            }

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma))
                || matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Semi))
            {
                if is_variadic {
                    return Err(Box::new(ParseError::from_token(
                        &sep,
                        "Variadic parameter must be the last parameter".to_string(),
                    )));
                }
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                return Ok(params);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ')' after parameter".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Parameter parsing exceeded safety bound".to_string(),
            file: None,
            line: 1,
            column: 1,
            length: 1,
        }))
    }

    pub(super) fn parse_routine_name_with_optional_receiver(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        missing_name_message: &str,
    ) -> Result<(Option<FolType>, String), Box<dyn Glitch>> {
        let mut receiver_type = None;
        let current = tokens.curr(false)?;

        if matches!(current.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let receiver_token = tokens.curr(false)?;
            if receiver_token.key().is_buildin() {
                return Err(Box::new(ParseError::from_token(
                    &receiver_token,
                    "Method receiver type must be a user-defined named type".to_string(),
                )));
            }

            receiver_type = Some(self.parse_type_reference_tokens(tokens)?);
            match receiver_type.as_ref() {
                Some(FolType::Named { name }) if !Self::is_builtin_scalar_type_name(name) => {}
                Some(FolType::Named { .. })
                | Some(FolType::Int { .. })
                | Some(FolType::Float { .. })
                | Some(FolType::Bool)
                | Some(FolType::Char { .. })
                | Some(FolType::Any)
                | Some(FolType::None) => {
                    return Err(Box::new(ParseError::from_token(
                        &receiver_token,
                        "Method receiver type must be a user-defined named type".to_string(),
                    )));
                }
                Some(_) => {}
                None => unreachable!("receiver_type is set above"),
            }

            self.skip_ignorable(tokens);
            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected ')' after method receiver type".to_string(),
                )));
            }
            let _ = tokens.bump();
            self.skip_ignorable(tokens);
        }

        let name_token = tokens.curr(false)?;
        let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &name_token,
                missing_name_message.to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();
        Ok((receiver_type, name))
    }

    pub(super) fn register_routine_return_type(
        &self,
        routine_name: &str,
        arity: usize,
        receiver_type: Option<&FolType>,
        return_type: &FolType,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<(), Box<dyn Glitch>> {
        self.register_routine_return_type_key(
            Self::callable_key(routine_name, arity),
            routine_name.to_string(),
            return_type,
            token,
        )?;

        if let Some(FolType::Named {
            name: receiver_name,
        }) = receiver_type
        {
            let qualified_name = format!("{}.{}", receiver_name, routine_name);
            self.register_routine_return_type_key(
                Self::callable_key(&qualified_name, arity),
                qualified_name,
                return_type,
                token,
            )?;
        }

        Ok(())
    }

    pub(super) fn register_routine_return_type_key(
        &self,
        key: String,
        label: String,
        return_type: &FolType,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<(), Box<dyn Glitch>> {
        let mut registry = self.routine_return_types.borrow_mut();
        if let Some(existing) = registry.get(&key) {
            if existing != return_type {
                return Err(Box::new(ParseError::from_token(
                    token,
                    format!(
                        "Conflicting return type for routine '{}': '{}' vs '{}'",
                        label,
                        Self::fol_type_label(existing),
                        Self::fol_type_label(return_type)
                    ),
                )));
            }
            return Ok(());
        }

        registry.insert(key, return_type.clone());
        Ok(())
    }

    pub(super) fn callable_key(name: &str, arity: usize) -> String {
        format!("{}#{}", name, arity)
    }

    pub(super) fn reported_callable_arity_mismatch_message(
        name: &str,
        arity: usize,
        routine_return_types: &HashMap<String, FolType>,
    ) -> Option<String> {
        let mut arities: Vec<usize> = routine_return_types
            .keys()
            .filter_map(|key| Self::parse_callable_key(key))
            .filter_map(|(candidate_name, candidate_arity)| {
                if candidate_name == name {
                    Some(candidate_arity)
                } else {
                    None
                }
            })
            .collect();

        if arities.is_empty() {
            return None;
        }

        arities.sort_unstable();
        arities.dedup();
        let available = arities
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        Some(format!(
            "Unknown reported callable '{}' with {} argument(s); available arity(s): {}",
            name, arity, available
        ))
    }

    pub(super) fn parse_callable_key(key: &str) -> Option<(String, usize)> {
        let (name, arity) = key.rsplit_once('#')?;
        let parsed_arity = arity.parse::<usize>().ok()?;
        Some((name.to_string(), parsed_arity))
    }

    pub(super) fn fol_type_label(typ: &FolType) -> String {
        match typ {
            FolType::Named { name } => name.clone(),
            FolType::Path { name } => {
                if name.is_empty() {
                    "path".to_string()
                } else {
                    name.clone()
                }
            }
            FolType::Url { name } => {
                if name.is_empty() {
                    "url".to_string()
                } else {
                    name.clone()
                }
            }
            FolType::Location { name } => {
                if name.is_empty() {
                    "loc".to_string()
                } else {
                    name.clone()
                }
            }
            FolType::Standard { name } => {
                if name.is_empty() {
                    "std".to_string()
                } else {
                    name.clone()
                }
            }
            _ => format!("{:?}", typ),
        }
    }

    pub(super) fn lower_bare_scalar_type_name(name: &str) -> Option<FolType> {
        match name {
            "int" => Some(FolType::Int {
                size: None,
                signed: true,
            }),
            "flt" | "float" => Some(FolType::Float { size: None }),
            "bol" | "bool" => Some(FolType::Bool),
            "chr" | "char" => Some(FolType::Char {
                encoding: CharEncoding::Utf8,
            }),
            "i8" => Some(FolType::Int {
                size: Some(IntSize::I8),
                signed: true,
            }),
            "i16" => Some(FolType::Int {
                size: Some(IntSize::I16),
                signed: true,
            }),
            "i32" => Some(FolType::Int {
                size: Some(IntSize::I32),
                signed: true,
            }),
            "i64" => Some(FolType::Int {
                size: Some(IntSize::I64),
                signed: true,
            }),
            "i128" => Some(FolType::Int {
                size: Some(IntSize::I128),
                signed: true,
            }),
            "u8" => Some(FolType::Int {
                size: Some(IntSize::I8),
                signed: false,
            }),
            "u16" => Some(FolType::Int {
                size: Some(IntSize::I16),
                signed: false,
            }),
            "u32" => Some(FolType::Int {
                size: Some(IntSize::I32),
                signed: false,
            }),
            "u64" => Some(FolType::Int {
                size: Some(IntSize::I64),
                signed: false,
            }),
            "u128" => Some(FolType::Int {
                size: Some(IntSize::I128),
                signed: false,
            }),
            "arch" => Some(FolType::Int {
                size: Some(IntSize::Arch),
                signed: true,
            }),
            "uarch" => Some(FolType::Int {
                size: Some(IntSize::Arch),
                signed: false,
            }),
            "f32" => Some(FolType::Float {
                size: Some(FloatSize::F32),
            }),
            "f64" => Some(FolType::Float {
                size: Some(FloatSize::F64),
            }),
            _ => None,
        }
    }

    pub(super) fn lower_integer_option(option: &str) -> Option<(IntSize, bool)> {
        match option {
            "8" | "i8" => Some((IntSize::I8, true)),
            "16" | "i16" => Some((IntSize::I16, true)),
            "32" | "i32" => Some((IntSize::I32, true)),
            "64" | "i64" => Some((IntSize::I64, true)),
            "128" | "i128" => Some((IntSize::I128, true)),
            "arch" => Some((IntSize::Arch, true)),
            "u8" => Some((IntSize::I8, false)),
            "u16" => Some((IntSize::I16, false)),
            "u32" => Some((IntSize::I32, false)),
            "u64" => Some((IntSize::I64, false)),
            "u128" => Some((IntSize::I128, false)),
            "uarch" => Some((IntSize::Arch, false)),
            _ => None,
        }
    }

    pub(super) fn lower_float_option(option: &str) -> Option<FloatSize> {
        match option {
            "32" | "f32" => Some(FloatSize::F32),
            "64" | "f64" => Some(FloatSize::F64),
            "arch" => Some(FloatSize::Arch),
            _ => None,
        }
    }

    pub(super) fn lower_char_option(option: &str) -> Option<CharEncoding> {
        match option {
            "utf8" => Some(CharEncoding::Utf8),
            "utf16" => Some(CharEncoding::Utf16),
            "utf32" => Some(CharEncoding::Utf32),
            _ => None,
        }
    }

    pub(super) fn parse_type_reference_tokens(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<FolType, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let token = tokens.curr(false)?;

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return self.parse_function_type_reference(tokens);
        }

        let mut name = Self::token_to_named_label(&token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &token,
                "Expected type reference".to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();

        for _ in 0..64 {
            self.skip_ignorable(tokens);
            let separator = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            let is_path = matches!(separator.key(), KEYWORD::Operator(OPERATOR::Path))
                || matches!(separator.key(), KEYWORD::Symbol(SYMBOL::Colon))
                    && matches!(
                        self.next_significant_key_from_window(tokens),
                        Some(KEYWORD::Symbol(SYMBOL::Colon))
                    );

            if !is_path {
                break;
            }

            if matches!(separator.key(), KEYWORD::Operator(OPERATOR::Path)) {
                let _ = tokens.bump();
            } else {
                self.consume_significant_token(tokens);
                self.consume_significant_token(tokens);
            }

            self.skip_ignorable(tokens);
            let segment = tokens.curr(false)?;
            let segment_name = Self::token_to_named_label(&segment).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &segment,
                    "Expected type segment after '::'".to_string(),
                )) as Box<dyn Glitch>
            })?;

            name.push_str("::");
            name.push_str(&segment_name);
            let _ = tokens.bump();
        }

        let base_name = name.clone();
        for _ in 0..32 {
            self.skip_ignorable(tokens);
            let open = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
                break;
            }

            if let Some(parsed) = self.try_parse_special_type_suffix(tokens, &base_name)? {
                return Ok(parsed);
            }

            name.push_str(&self.parse_balanced_type_suffix(
                tokens,
                KEYWORD::Symbol(SYMBOL::SquarO),
                KEYWORD::Symbol(SYMBOL::SquarC),
                "Expected closing ']' in type reference",
            )?);
        }

        if name == "mod" {
            return Ok(FolType::Module {
                name: String::new(),
            });
        }

        if name == "blk" {
            return Ok(FolType::Block {
                name: String::new(),
            });
        }

        if name == "any" {
            return Ok(FolType::Any);
        }

        if matches!(name.as_str(), "non" | "none") {
            return Ok(FolType::None);
        }

        if let Some(lowered) = Self::lower_bare_scalar_type_name(&name) {
            return Ok(lowered);
        }

        if let Some(lowered) = Self::lower_bare_source_kind_type_name(&name) {
            return Ok(lowered);
        }

        Ok(FolType::Named { name })
    }
}
