use super::*;

impl AstParser {
    pub(super) fn parse_primary(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        if matches!(token.key(), KEYWORD::Keyword(BUILDIN::True)) {
            return Ok(AstNode::Literal(Literal::Boolean(true)));
        }

        if matches!(token.key(), KEYWORD::Keyword(BUILDIN::False)) {
            return Ok(AstNode::Literal(Literal::Boolean(false)));
        }

        if token.key().is_literal() {
            return self.parse_lexer_literal(token);
        }

        if token.key().is_ident() {
            return Ok(AstNode::Identifier {
                name: token.con().trim().to_string(),
            });
        }

        Err(Box::new(ParseError::from_token(
            token,
            format!("Unsupported expression token '{}'", token.con()),
        )))
    }

    pub(super) fn skip_ignorable(&self, tokens: &mut fol_lexer::lexer::stage3::Elements) {
        for _ in 0..128 {
            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if token.key().is_void() || token.key().is_comment() {
                if tokens.bump().is_none() {
                    break;
                }
                continue;
            }

            break;
        }
    }

    pub(super) fn unary_prefix_info(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Option<(&'static str, Option<UnaryOperator>)> {
        if matches!(
            token.key(),
            KEYWORD::Operator(OPERATOR::Abstract) | KEYWORD::Symbol(SYMBOL::Minus)
        ) {
            return Some((
                "Expected expression after unary '-'",
                Some(UnaryOperator::Neg),
            ));
        }

        if matches!(
            token.key(),
            KEYWORD::Operator(OPERATOR::Add) | KEYWORD::Symbol(SYMBOL::Plus)
        ) {
            return Some(("Expected expression after unary '+'", None));
        }

        if self.token_is_word(token, "not") {
            return Some((
                "Expected expression after unary 'not'",
                Some(UnaryOperator::Not),
            ));
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::And)) {
            return Some((
                "Expected expression after unary '&'",
                Some(UnaryOperator::Ref),
            ));
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Star)) {
            return Some((
                "Expected expression after unary '*'",
                Some(UnaryOperator::Deref),
            ));
        }

        None
    }

    pub(super) fn ensure_unary_operand(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        operator_token: &fol_lexer::lexer::stage3::element::Element,
        message: &str,
    ) -> Result<(), Box<dyn Glitch>> {
        self.skip_ignorable(tokens);

        match tokens.curr(false) {
            Ok(next) => {
                if next.key().is_void()
                    || matches!(
                        next.key(),
                        KEYWORD::Symbol(SYMBOL::Semi)
                            | KEYWORD::Symbol(SYMBOL::Comma)
                            | KEYWORD::Symbol(SYMBOL::RoundC)
                            | KEYWORD::Symbol(SYMBOL::CurlyC)
                    )
                {
                    return Err(Box::new(ParseError::from_token(&next, message.to_string())));
                }

                Ok(())
            }
            Err(_) => Err(Box::new(ParseError::from_token(
                operator_token,
                message.to_string(),
            ))),
        }
    }

    pub(super) fn validate_report_usage(
        nodes: &[AstNode],
        routine_error_type: Option<&FolType>,
        visible_types: &HashMap<String, FolType>,
        routine_return_types: &HashMap<String, FolType>,
        routine_token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Result<(), Box<dyn Glitch>> {
        if routine_error_type.is_none() {
            return Ok(());
        }

        let mut scope_types = visible_types.clone();

        for node in nodes {
            match node {
                AstNode::VarDecl {
                    name,
                    type_hint,
                    value,
                    ..
                } => {
                    if let Some(typ) = type_hint.clone() {
                        scope_types.insert(name.clone(), typ);
                    } else if let Some(val) = value {
                        if let Some(inferred) = Self::infer_named_type_from_node(
                            val.as_ref(),
                            &scope_types,
                            routine_return_types,
                        ) {
                            scope_types.insert(name.clone(), inferred);
                        }
                    }
                }
                AstNode::FunctionCall { name, args } if name == "report" => {
                    if args.len() != 1 {
                        return Err(Box::new(ParseError::from_token(
                            routine_token,
                            "Routine with custom error type must report exactly one error value"
                                .to_string(),
                        )));
                    }

                    if let Some(expected_type) = routine_error_type {
                        if let Some(unknown_identifier) =
                            Self::report_unknown_identifier_in_expression(
                                &args[0],
                                &scope_types,
                                routine_return_types,
                            )
                        {
                            return Err(Box::new(ParseError::from_token(
                                routine_token,
                                unknown_identifier,
                            )));
                        }

                        if let Some(mismatch) = Self::report_identifier_type_mismatch(
                            &args[0],
                            expected_type,
                            &scope_types,
                        ) {
                            return Err(Box::new(ParseError::from_token(routine_token, mismatch)));
                        }

                        if let Some(mismatch) =
                            Self::report_literal_type_mismatch(&args[0], expected_type)
                        {
                            return Err(Box::new(ParseError::from_token(routine_token, mismatch)));
                        }

                        if let Some(mismatch) = Self::report_expression_type_mismatch(
                            &args[0],
                            expected_type,
                            &scope_types,
                            routine_return_types,
                        ) {
                            return Err(Box::new(ParseError::from_token(routine_token, mismatch)));
                        }
                    }
                }
                AstNode::When { cases, default, .. } => {
                    for case in cases {
                        match case {
                            WhenCase::Case { body, .. }
                            | WhenCase::Is { body, .. }
                            | WhenCase::In { body, .. }
                            | WhenCase::Has { body, .. }
                            | WhenCase::Of { body, .. }
                            | WhenCase::On { body, .. } => {
                                Self::validate_report_usage(
                                    body,
                                    routine_error_type,
                                    &scope_types,
                                    routine_return_types,
                                    routine_token,
                                )?;
                            }
                        }
                    }
                    if let Some(default_body) = default {
                        Self::validate_report_usage(
                            default_body,
                            routine_error_type,
                            &scope_types,
                            routine_return_types,
                            routine_token,
                        )?;
                    }
                }
                AstNode::Loop { body, .. } => {
                    Self::validate_report_usage(
                        body,
                        routine_error_type,
                        &scope_types,
                        routine_return_types,
                        routine_token,
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    pub(super) fn report_literal_type_mismatch(value: &AstNode, expected_type: &FolType) -> Option<String> {
        let expected_name = Self::type_family_name(expected_type)?;

        if !Self::is_builtin_scalar_type_name(&expected_name) {
            return None;
        }

        let literal = match value {
            AstNode::Literal(lit) => lit,
            _ => return None,
        };

        if Self::literal_matches_named_type(literal, &expected_name) {
            None
        } else {
            Some(format!(
                "Reported literal value is incompatible with routine error type '{}'",
                expected_name
            ))
        }
    }

    pub(super) fn report_identifier_type_mismatch(
        value: &AstNode,
        expected_type: &FolType,
        visible_types: &HashMap<String, FolType>,
    ) -> Option<String> {
        let expected_name = Self::type_family_name(expected_type)?;

        if !Self::is_builtin_scalar_type_name(&expected_name) {
            return None;
        }

        let identifier_name = match value {
            AstNode::Identifier { name } => name,
            _ => return None,
        };

        let found_type_name = Self::type_family_name(visible_types.get(identifier_name)?)?;

        if Self::named_types_compatible(&found_type_name, &expected_name) {
            None
        } else {
            Some(format!(
                "Reported identifier '{}' has type '{}' incompatible with routine error type '{}'",
                identifier_name, found_type_name, expected_name
            ))
        }
    }

    pub(super) fn report_unknown_identifier_in_expression(
        value: &AstNode,
        visible_types: &HashMap<String, FolType>,
        routine_return_types: &HashMap<String, FolType>,
    ) -> Option<String> {
        match value {
            AstNode::Identifier { name } => {
                if visible_types.contains_key(name) {
                    None
                } else {
                    Some(format!(
                        "Unknown reported identifier '{}' in custom-error routine",
                        name
                    ))
                }
            }
            AstNode::BinaryOp { left, right, .. } => Self::report_unknown_identifier_in_expression(
                left,
                visible_types,
                routine_return_types,
            )
            .or_else(|| {
                Self::report_unknown_identifier_in_expression(
                    right,
                    visible_types,
                    routine_return_types,
                )
            }),
            AstNode::UnaryOp { operand, .. } => Self::report_unknown_identifier_in_expression(
                operand,
                visible_types,
                routine_return_types,
            ),
            AstNode::FunctionCall { name, args } => {
                let callable_key = Self::callable_key(name, args.len());
                if !routine_return_types.contains_key(&callable_key) {
                    if let Some(arity_message) = Self::reported_callable_arity_mismatch_message(
                        name,
                        args.len(),
                        routine_return_types,
                    ) {
                        return Some(arity_message);
                    }

                    return Some(format!(
                        "Unknown reported routine '{}' in custom-error routine",
                        name
                    ));
                }

                args.iter().find_map(|arg| {
                    Self::report_unknown_identifier_in_expression(
                        arg,
                        visible_types,
                        routine_return_types,
                    )
                })
            }
            AstNode::MethodCall { object, args, .. } => {
                if let AstNode::MethodCall { object, method, .. } = value {
                    if let Some(FolType::Named { name: object_type }) =
                        Self::infer_named_type_from_node(
                            object,
                            visible_types,
                            routine_return_types,
                        )
                    {
                        let qualified_method = format!("{}.{}", object_type, method);
                        let qualified_method_name =
                            Self::callable_key(&qualified_method, args.len());
                        if !routine_return_types.contains_key(&qualified_method_name) {
                            if let Some(arity_message) =
                                Self::reported_callable_arity_mismatch_message(
                                    &qualified_method,
                                    args.len(),
                                    routine_return_types,
                                )
                            {
                                return Some(arity_message);
                            }

                            return Some(format!(
                                "Unknown reported method '{}.{}' in custom-error routine",
                                object_type, method
                            ));
                        }
                    }
                }

                Self::report_unknown_identifier_in_expression(
                    object,
                    visible_types,
                    routine_return_types,
                )
                .or_else(|| {
                    args.iter().find_map(|arg| {
                        Self::report_unknown_identifier_in_expression(
                            arg,
                            visible_types,
                            routine_return_types,
                        )
                    })
                })
            }
            AstNode::IndexAccess { container, index } => {
                Self::report_unknown_identifier_in_expression(
                    container,
                    visible_types,
                    routine_return_types,
                )
                .or_else(|| {
                    Self::report_unknown_identifier_in_expression(
                        index,
                        visible_types,
                        routine_return_types,
                    )
                })
            }
            AstNode::FieldAccess { object, .. } => Self::report_unknown_identifier_in_expression(
                object,
                visible_types,
                routine_return_types,
            ),
            _ => None,
        }
    }

    pub(super) fn report_expression_type_mismatch(
        value: &AstNode,
        expected_type: &FolType,
        visible_types: &HashMap<String, FolType>,
        routine_return_types: &HashMap<String, FolType>,
    ) -> Option<String> {
        let expected_name = Self::type_family_name(expected_type)?;

        if !Self::is_builtin_scalar_type_name(&expected_name) {
            return None;
        }

        let found_name =
            Self::infer_named_type_from_node(value, visible_types, routine_return_types)?;
        let found = Self::type_family_name(&found_name)?;

        if Self::named_types_compatible(&found, &expected_name) {
            None
        } else {
            Some(format!(
                "Reported expression type '{}' is incompatible with routine error type '{}'",
                found, expected_name
            ))
        }
    }

    pub(super) fn literal_matches_named_type(literal: &Literal, expected_name: &str) -> bool {
        match literal {
            Literal::Integer(_) => matches!(
                expected_name,
                "int" | "num" | "i8" | "i16" | "i32" | "i64" | "i128"
            ),
            Literal::Float(_) => matches!(expected_name, "flt" | "float" | "num" | "f32" | "f64"),
            Literal::Boolean(_) => matches!(expected_name, "bol" | "bool"),
            Literal::String(_) => matches!(expected_name, "str"),
            Literal::Character(_) => matches!(expected_name, "chr" | "char"),
        }
    }

    pub(super) fn is_builtin_scalar_type_name(name: &str) -> bool {
        matches!(
            name,
            "int"
                | "num"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "flt"
                | "float"
                | "f32"
                | "f64"
                | "bol"
                | "bool"
                | "str"
                | "chr"
                | "char"
        )
    }

    pub(super) fn parameter_type_map(params: &[Parameter]) -> HashMap<String, FolType> {
        params
            .iter()
            .map(|parameter| (parameter.name.clone(), parameter.param_type.clone()))
            .collect()
    }

    pub(super) fn infer_named_type_from_node(
        node: &AstNode,
        visible_types: &HashMap<String, FolType>,
        routine_return_types: &HashMap<String, FolType>,
    ) -> Option<FolType> {
        match node {
            AstNode::Identifier { name } => {
                let found = visible_types.get(name)?;
                Self::fol_type_to_named_family(found.clone())
            }
            AstNode::FunctionCall { name, args } => {
                let found = routine_return_types.get(&Self::callable_key(name, args.len()))?;
                Self::fol_type_to_named_family(found.clone())
            }
            AstNode::MethodCall {
                object,
                method,
                args,
            } => {
                if let Some(FolType::Named { name: object_type }) =
                    Self::infer_named_type_from_node(object, visible_types, routine_return_types)
                {
                    let qualified_method_name =
                        Self::callable_key(&format!("{}.{}", object_type, method), args.len());
                    let found = routine_return_types.get(&qualified_method_name)?;
                    return Self::fol_type_to_named_family(found.clone());
                }

                None
            }
            AstNode::Literal(Literal::String(_)) => Some(FolType::Named {
                name: "str".to_string(),
            }),
            AstNode::Literal(Literal::Boolean(_)) => Some(FolType::Named {
                name: "bol".to_string(),
            }),
            AstNode::Literal(Literal::Integer(_)) => Some(FolType::Named {
                name: "int".to_string(),
            }),
            AstNode::Literal(Literal::Float(_)) => Some(FolType::Named {
                name: "flt".to_string(),
            }),
            AstNode::Literal(Literal::Character(_)) => Some(FolType::Named {
                name: "chr".to_string(),
            }),
            AstNode::BinaryOp { left, right, .. } => {
                let left_type =
                    Self::infer_named_type_from_node(left, visible_types, routine_return_types);
                let right_type =
                    Self::infer_named_type_from_node(right, visible_types, routine_return_types);

                match (left_type, right_type) {
                    (Some(FolType::Named { name: l }), Some(FolType::Named { name: r })) => {
                        if Self::is_numeric_named_type(&l) && Self::is_numeric_named_type(&r) {
                            Some(FolType::Named {
                                name: if l == "num" || r == "num" {
                                    "num".to_string()
                                } else {
                                    l
                                },
                            })
                        } else if l == r {
                            Some(FolType::Named { name: l })
                        } else {
                            None
                        }
                    }
                    (Some(t), None) | (None, Some(t)) => Some(t),
                    _ => None,
                }
            }
            AstNode::UnaryOp { operand, .. } => {
                Self::infer_named_type_from_node(operand, visible_types, routine_return_types)
            }
            _ => Self::fol_type_to_named_family(node.get_type()?),
        }
    }

    pub(super) fn fol_type_to_named_family(typ: FolType) -> Option<FolType> {
        match typ {
            FolType::Int { .. } => Some(FolType::Named {
                name: "int".to_string(),
            }),
            FolType::Float { .. } => Some(FolType::Named {
                name: "flt".to_string(),
            }),
            FolType::Bool => Some(FolType::Named {
                name: "bol".to_string(),
            }),
            FolType::Char { .. } => Some(FolType::Named {
                name: "chr".to_string(),
            }),
            FolType::Named { name } => Some(FolType::Named { name }),
            _ => None,
        }
    }

    pub(super) fn type_family_name(typ: &FolType) -> Option<String> {
        match Self::fol_type_to_named_family(typ.clone())? {
            FolType::Named { name } => Some(name),
            _ => None,
        }
    }

    pub(super) fn named_types_compatible(found_name: &str, expected_name: &str) -> bool {
        if found_name == expected_name {
            return true;
        }

        let found_numeric = Self::is_numeric_named_type(found_name);
        let expected_numeric = Self::is_numeric_named_type(expected_name);
        if found_numeric && expected_numeric {
            return true;
        }

        match found_name {
            "bool" => matches!(expected_name, "bol"),
            "bol" => matches!(expected_name, "bool"),
            "float" => matches!(expected_name, "flt"),
            "flt" => matches!(expected_name, "float"),
            "char" => matches!(expected_name, "chr"),
            "chr" => matches!(expected_name, "char"),
            _ => false,
        }
    }

    pub(super) fn is_numeric_named_type(name: &str) -> bool {
        matches!(
            name,
            "num" | "int" | "flt" | "float" | "i8" | "i16" | "i32" | "i64" | "i128" | "f32" | "f64"
        )
    }

    pub(super) fn consume_optional_semicolon(&self, tokens: &mut fol_lexer::lexer::stage3::Elements) {
        self.skip_ignorable(tokens);
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Semi)) {
                let _ = tokens.bump();
            }
        }
    }

    /// Parse a simple literal for testing
    pub fn parse_literal(&self, value: &str) -> Result<AstNode, Box<dyn Glitch>> {
        // Simple integer parsing for testing
        if let Ok(int_val) = value.parse::<i64>() {
            return Ok(AstNode::Literal(Literal::Integer(int_val)));
        }

        // Simple string parsing
        if value.starts_with('"') && value.ends_with('"') {
            let string_val = value[1..value.len() - 1].to_string();
            return Ok(AstNode::Literal(Literal::String(string_val)));
        }

        // Default to identifier
        Ok(AstNode::Identifier {
            name: value.to_string(),
        })
    }
}
