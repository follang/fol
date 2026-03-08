use super::*;

impl AstParser {
    pub(super) fn parse_function_type_reference(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<FolType, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start function type".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let fun_token = tokens.curr(false)?;
        if !matches!(fun_token.key(), KEYWORD::Keyword(BUILDIN::Fun)) {
            return Err(Box::new(ParseError::from_token(
                &fun_token,
                "Expected 'fun' in function type".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        if let Ok(token) = tokens.curr(false) {
            if token.key().is_ident() {
                let _ = tokens.bump();
            }
        }

        self.skip_ignorable(tokens);
        let open_params = tokens.curr(false)?;
        if !matches!(open_params.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            return Err(Box::new(ParseError::from_token(
                &open_params,
                "Expected '(' in function type".to_string(),
            )));
        }
        let _ = tokens.bump();

        let params = self.parse_parameter_list(tokens)?;
        if params.iter().any(|param| param.default.is_some()) {
            return Err(Box::new(ParseError::from_token(
                &fun_token,
                "Default values are not allowed in function types".to_string(),
            )));
        }

        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' before function type return type".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let return_type = self.parse_type_reference_tokens(tokens)?;

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected '}' to close function type".to_string(),
            )));
        }
        let _ = tokens.bump();

        Ok(FolType::Function {
            params: params.into_iter().map(|param| param.param_type).collect(),
            return_type: Box::new(return_type),
        })
    }

    pub(super) fn try_parse_special_type_suffix(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        base_name: &str,
    ) -> Result<Option<FolType>, Box<dyn Glitch>> {
        match base_name {
            "opt" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() != 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected exactly one type argument for opt[...]".to_string(),
                    )));
                }
                Ok(Some(FolType::Optional {
                    inner: Box::new(args.into_iter().next().expect("opt arg exists")),
                }))
            }
            "mul" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.is_empty() {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected at least one type argument for mul[...]".to_string(),
                    )));
                }
                Ok(Some(FolType::Multiple { types: args }))
            }
            "ptr" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() != 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected exactly one type argument for ptr[...]".to_string(),
                    )));
                }
                Ok(Some(FolType::Pointer {
                    target: Box::new(args.into_iter().next().expect("ptr arg exists")),
                }))
            }
            "err" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() > 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected zero or one type argument for err[...]".to_string(),
                    )));
                }
                Ok(Some(FolType::Error {
                    inner: args.into_iter().next().map(Box::new),
                }))
            }
            "vec" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() != 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected exactly one type argument for vec[...]".to_string(),
                    )));
                }
                Ok(Some(FolType::Vector {
                    element_type: Box::new(args.into_iter().next().expect("vec arg exists")),
                }))
            }
            "arr" => {
                let (element_type, size) = self.parse_array_type_arguments(tokens)?;
                Ok(Some(FolType::Array {
                    element_type: Box::new(element_type),
                    size: Some(size),
                }))
            }
            "mat" => {
                let (element_type, dimensions) = self.parse_matrix_type_arguments(tokens)?;
                Ok(Some(FolType::Matrix {
                    element_type: Box::new(element_type),
                    dimensions,
                }))
            }
            "seq" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() != 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected exactly one type argument for seq[...]".to_string(),
                    )));
                }
                Ok(Some(FolType::Sequence {
                    element_type: Box::new(args.into_iter().next().expect("seq arg exists")),
                }))
            }
            "set" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.is_empty() {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected at least one type argument for set[...]".to_string(),
                    )));
                }
                Ok(Some(FolType::Set { types: args }))
            }
            "map" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() != 2 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected exactly two type arguments for map[...]".to_string(),
                    )));
                }
                let mut args = args.into_iter();
                Ok(Some(FolType::Map {
                    key_type: Box::new(args.next().expect("map key exists")),
                    value_type: Box::new(args.next().expect("map value exists")),
                }))
            }
            "mod" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() > 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected zero or one type argument for mod[...]".to_string(),
                    )));
                }
                let name = match args.into_iter().next() {
                    None => String::new(),
                    Some(FolType::Named { name }) => name,
                    Some(other) => Self::fol_type_label(&other),
                };
                Ok(Some(FolType::Module { name }))
            }
            "blk" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() > 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected zero or one type argument for blk[...]".to_string(),
                    )));
                }
                let name = match args.into_iter().next() {
                    None => String::new(),
                    Some(FolType::Named { name }) => name,
                    Some(other) => Self::fol_type_label(&other),
                };
                Ok(Some(FolType::Block { name }))
            }
            "int" => Ok(Some(self.parse_integer_type_reference(tokens)?)),
            "flt" | "float" => Ok(Some(self.parse_float_type_reference(tokens)?)),
            "chr" | "char" => Ok(Some(self.parse_char_type_reference(tokens)?)),
            _ => Ok(None),
        }
    }

    pub(super) fn parse_integer_type_reference(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<FolType, Box<dyn Glitch>> {
        let args = self
            .parse_scalar_type_options(tokens, "Expected closing ']' in integer type reference")?;

        if args.len() != 1 {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected exactly one integer type option in int[...]".to_string(),
            )));
        }

        let Some((size, signed)) = Self::lower_integer_option(&args[0]) else {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                format!("Unknown integer type option '{}'", args[0]),
            )));
        };

        Ok(FolType::Int {
            size: Some(size),
            signed,
        })
    }

    pub(super) fn parse_float_type_reference(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<FolType, Box<dyn Glitch>> {
        let args =
            self.parse_scalar_type_options(tokens, "Expected closing ']' in float type reference")?;

        if args.len() != 1 {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected exactly one float type option in flt[...]".to_string(),
            )));
        }

        let Some(size) = Self::lower_float_option(&args[0]) else {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                format!("Unknown float type option '{}'", args[0]),
            )));
        };

        Ok(FolType::Float { size: Some(size) })
    }

    pub(super) fn parse_char_type_reference(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<FolType, Box<dyn Glitch>> {
        let args = self.parse_scalar_type_options(
            tokens,
            "Expected closing ']' in character type reference",
        )?;

        if args.len() != 1 {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected exactly one character encoding in chr[...]".to_string(),
            )));
        }

        let Some(encoding) = Self::lower_char_option(&args[0]) else {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                format!("Unknown character type option '{}'", args[0]),
            )));
        };

        Ok(FolType::Char { encoding })
    }

    pub(super) fn parse_scalar_type_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        missing_close_message: &str,
    ) -> Result<Vec<String>, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '[' to start scalar type options".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut args = Vec::new();
        for _ in 0..16 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(args);
            }

            let option =
                if token.key().is_ident() || token.key().is_buildin() || token.key().is_number() {
                    token.con().trim().to_string()
                } else {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected scalar type option".to_string(),
                    )));
                };
            args.push(option);
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(args);
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                missing_close_message.to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Scalar type option list exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }

    pub(super) fn parse_type_argument_list(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<FolType>, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '[' to start type argument list".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut args = Vec::new();
        for _ in 0..64 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(args);
            }

            args.push(self.parse_type_reference_tokens(tokens)?);
            self.skip_ignorable(tokens);

            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                let _ = tokens.bump();
                return Ok(args);
            }
            if sep.key().is_terminal() || matches!(sep.key(), KEYWORD::Void(_)) {
                return Err(Box::new(ParseError::from_token(
                    &sep,
                    "Expected closing ']' in type reference".to_string(),
                )));
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected closing ']' in type reference".to_string(),
            )));
        }

        Err(Box::new(ParseError {
            message: "Type argument list exceeded parser limit".to_string(),
            file: None,
            line: 0,
            column: 0,
            length: 0,
        }))
    }

    pub(super) fn parse_array_type_arguments(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<(FolType, usize), Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '[' to start array type arguments".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let element_type = self.parse_type_reference_tokens(tokens)?;
        self.skip_ignorable(tokens);

        let comma = tokens.curr(false)?;
        if !matches!(comma.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
            return Err(Box::new(ParseError::from_token(
                &comma,
                "Expected ',' after array element type".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let size_token = tokens.curr(false)?;
        let size = match size_token.key() {
            KEYWORD::Literal(LITERAL::Deciaml) => {
                size_token.con().trim().parse::<usize>().map_err(|_| {
                    Box::new(ParseError::from_token(
                        &size_token,
                        "Expected decimal array size in arr[...]".to_string(),
                    )) as Box<dyn Glitch>
                })?
            }
            _ => {
                return Err(Box::new(ParseError::from_token(
                    &size_token,
                    "Expected decimal array size in arr[...]".to_string(),
                )))
            }
        };
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected closing ']' in type reference".to_string(),
            )));
        }
        let _ = tokens.bump();

        Ok((element_type, size))
    }

    pub(super) fn parse_matrix_type_arguments(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<(FolType, Vec<usize>), Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '[' to start matrix type arguments".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let element_type = self.parse_type_reference_tokens(tokens)?;
        let mut dimensions = Vec::new();

        for _ in 0..8 {
            self.skip_ignorable(tokens);
            let comma = tokens.curr(false)?;
            if matches!(comma.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
                break;
            }
            if !matches!(comma.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                return Err(Box::new(ParseError::from_token(
                    &comma,
                    "Expected ',' after matrix element type".to_string(),
                )));
            }
            let _ = tokens.bump();

            self.skip_ignorable(tokens);
            let dim_token = tokens.curr(false)?;
            let dim = match dim_token.key() {
                KEYWORD::Literal(LITERAL::Deciaml) => {
                    dim_token.con().trim().parse::<usize>().map_err(|_| {
                        Box::new(ParseError::from_token(
                            &dim_token,
                            "Expected decimal matrix dimension in mat[...]".to_string(),
                        )) as Box<dyn Glitch>
                    })?
                }
                _ => {
                    return Err(Box::new(ParseError::from_token(
                        &dim_token,
                        "Expected decimal matrix dimension in mat[...]".to_string(),
                    )))
                }
            };
            dimensions.push(dim);
            let _ = tokens.bump();
        }

        if dimensions.is_empty() {
            let token = tokens.curr(false)?;
            return Err(Box::new(ParseError::from_token(
                &token,
                "Expected at least one matrix dimension in mat[...]".to_string(),
            )));
        }

        self.skip_ignorable(tokens);
        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Expected closing ']' in type reference".to_string(),
            )));
        }
        let _ = tokens.bump();

        Ok((element_type, dimensions))
    }

    pub(super) fn parse_balanced_type_suffix(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        open_key: KEYWORD,
        close_key: KEYWORD,
        missing_close_message: &str,
    ) -> Result<String, Box<dyn Glitch>> {
        let mut depth = 0usize;
        let mut rendered = String::new();
        let mut anchor_token = None;

        for _ in 0..512 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;
            let key = token.key();

            if key == open_key {
                if anchor_token.is_none() {
                    anchor_token = Some(token.clone());
                }
                depth += 1;
            } else if key == close_key {
                if depth == 0 {
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        missing_close_message.to_string(),
                    )));
                }
                depth -= 1;
            }

            let fragment = token.con().trim();
            if !fragment.is_empty() {
                rendered.push_str(fragment);
            }
            let _ = tokens.bump();

            if depth == 0 {
                return Ok(rendered);
            }
        }

        let token = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &token,
            missing_close_message.to_string(),
        )))
    }

    pub(super) fn parse_block_body(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        missing_close_message: &str,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut body = Vec::new();
        let mut anchor_token = None;

        for _ in 0..8_192 {
            self.skip_ignorable(tokens);

            let token = tokens.curr(false)?;
            if anchor_token.is_none() {
                anchor_token = Some(token.clone());
            }
            let key = token.key();

            if matches!(key, KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                return Ok(body);
            }

            if key.is_eof() {
                let anchor = anchor_token.unwrap_or(token);
                return Err(Box::new(ParseError::from_token(
                    &anchor,
                    missing_close_message.to_string(),
                )));
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Return)) {
                body.push(self.parse_return_stmt(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Break)) {
                body.push(self.parse_break_stmt(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Yeild)) {
                body.push(self.parse_yield_stmt(tokens)?);
                continue;
            }

            if matches!(
                key,
                KEYWORD::Keyword(BUILDIN::Panic)
                    | KEYWORD::Keyword(BUILDIN::Report)
                    | KEYWORD::Keyword(BUILDIN::Check)
                    | KEYWORD::Keyword(BUILDIN::Assert)
            ) {
                body.push(self.parse_builtin_call_stmt(tokens)?);
                continue;
            }

            if self.lookahead_binding_alternative(tokens).is_some() {
                body.extend(self.parse_binding_alternative_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Var)) {
                body.extend(self.parse_var_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Let)) {
                body.extend(self.parse_let_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Con)) {
                body.extend(self.parse_con_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Lab)) {
                body.extend(self.parse_lab_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Use)) {
                body.extend(self.parse_use_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Seg)) {
                body.push(self.parse_seg_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Imp)) {
                body.push(self.parse_imp_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Ali)) {
                body.push(self.parse_alias_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Typ)) {
                body.push(self.parse_type_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Def)) {
                body.push(self.parse_def_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Fun)) {
                body.push(self.parse_fun_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Log)) {
                body.push(self.parse_log_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::Pro)) {
                body.push(self.parse_pro_decl(tokens)?);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::When)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                body.push(self.parse_when_stmt(tokens)?);
                self.bump_if_no_progress(tokens, before);
                continue;
            }

            if matches!(key, KEYWORD::Keyword(BUILDIN::If)) {
                let before = (
                    token.loc().row(),
                    token.loc().col(),
                    token.con().to_string(),
                );
                body.push(self.parse_if_stmt(tokens)?);
                self.bump_if_no_progress(tokens, before);
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
                body.push(self.parse_loop_stmt(tokens)?);
                self.bump_if_no_progress(tokens, before);
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::CurlyO)) {
                body.push(self.parse_block_stmt(tokens)?);
                continue;
            }

            if AstParser::token_can_be_logical_name(&key)
                && self.lookahead_is_assignment(tokens)
                && self.can_start_assignment(tokens)
            {
                body.push(self.parse_assignment_stmt(tokens)?);
                continue;
            }

            if AstParser::token_can_be_logical_name(&key)
                && (self.lookahead_is_call(tokens) || self.lookahead_is_method_call(tokens))
                && self.can_start_assignment(tokens)
            {
                body.push(self.parse_call_stmt(tokens)?);
                continue;
            }

            if AstParser::token_can_be_logical_name(&key) {
                body.push(AstNode::Identifier {
                    name: token.con().trim().to_string(),
                });
            } else if key.is_literal() {
                body.push(self.parse_lexer_literal(&token)?);
            }

            if tokens.bump().is_none() {
                break;
            }
        }

        let anchor = match anchor_token {
            Some(token) => token,
            None => tokens.curr(false)?,
        };
        Err(Box::new(ParseError::from_token(
            &anchor,
            missing_close_message.to_string(),
        )))
    }

    pub(super) fn parse_block_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start block".to_string(),
            )));
        }
        let _ = tokens.bump();
        let statements = self.parse_block_body(tokens, "Expected '}' to close block")?;
        Ok(AstNode::Block { statements })
    }

    pub(super) fn parse_return_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        if tokens.bump().is_none() {
            return Ok(AstNode::Return { value: None });
        }

        self.skip_ignorable(tokens);

        let value = match tokens.curr(false) {
            Ok(token) if token.key().is_terminal() => None,
            Ok(_) => Some(Box::new(self.parse_logical_expression(tokens)?)),
            Err(_) => None,
        };

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::Return { value })
    }

    pub(super) fn parse_break_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let break_token = tokens.curr(false)?;
        if !matches!(break_token.key(), KEYWORD::Keyword(BUILDIN::Break)) {
            return Err(Box::new(ParseError::from_token(
                &break_token,
                "Expected 'break' statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.consume_optional_semicolon(tokens);

        Ok(AstNode::Break)
    }

    pub(super) fn parse_yield_stmt(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let yield_token = tokens.curr(false)?;
        if !matches!(yield_token.key(), KEYWORD::Keyword(BUILDIN::Yeild)) {
            return Err(Box::new(ParseError::from_token(
                &yield_token,
                "Expected 'yield' statement".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        let value = self.parse_logical_expression(tokens)?;

        self.consume_optional_semicolon(tokens);

        Ok(AstNode::Yield {
            value: Box::new(value),
        })
    }
}
