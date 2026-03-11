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

        if token.key().is_ident() && token.con().trim() == "nil" {
            return Ok(AstNode::Literal(Literal::Nil));
        }

        if Self::token_can_be_logical_name(&token.key()) {
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

    pub(super) fn token_can_be_logical_name(key: &KEYWORD) -> bool {
        key.is_ident()
            || matches!(
                key,
                KEYWORD::Keyword(BUILDIN::Use)
                    | KEYWORD::Keyword(BUILDIN::Def)
                    | KEYWORD::Keyword(BUILDIN::Seg)
                    | KEYWORD::Keyword(BUILDIN::Var)
                    | KEYWORD::Keyword(BUILDIN::Log)
                    | KEYWORD::Keyword(BUILDIN::Con)
                    | KEYWORD::Keyword(BUILDIN::Fun)
                    | KEYWORD::Keyword(BUILDIN::Pro)
                    | KEYWORD::Keyword(BUILDIN::Typ)
                    | KEYWORD::Keyword(BUILDIN::Std)
                    | KEYWORD::Keyword(BUILDIN::Ali)
                    | KEYWORD::Keyword(BUILDIN::Imp)
                    | KEYWORD::Keyword(BUILDIN::Lab)
                    | KEYWORD::Keyword(BUILDIN::This)
                    | KEYWORD::Keyword(BUILDIN::Selfi)
                    | KEYWORD::Keyword(BUILDIN::If)
                    | KEYWORD::Keyword(BUILDIN::Go)
                    | KEYWORD::Keyword(BUILDIN::Get)
                    | KEYWORD::Keyword(BUILDIN::Let)
                    | KEYWORD::Keyword(BUILDIN::Check)
                    | KEYWORD::Keyword(BUILDIN::Panic)
                    | KEYWORD::Keyword(BUILDIN::Report)
                    | KEYWORD::Keyword(BUILDIN::Assert)
            )
    }

    pub(super) fn token_to_named_label(
        token: &fol_lexer::lexer::stage3::element::Element,
    ) -> Option<String> {
        if Self::token_can_be_logical_name(&token.key()) {
            return Some(token.con().trim().to_string());
        }

        match token.key() {
            KEYWORD::Literal(LITERAL::Stringy) | KEYWORD::Literal(LITERAL::Quoted) => {
                Some(Self::exact_unquote_text(token.con()))
            }
            _ => None,
        }
    }

    pub(super) fn expect_named_label(
        token: &fol_lexer::lexer::stage3::element::Element,
        message: &str,
    ) -> Result<String, Box<dyn Glitch>> {
        if token.key().is_illegal() {
            return Err(Box::new(ParseError::from_token(
                token,
                format!("Parser encountered illegal token '{}'", token.con()),
            )));
        }

        Self::token_to_named_label(token).ok_or_else(|| {
            Box::new(ParseError::from_token(token, message.to_string())) as Box<dyn Glitch>
        })
    }

    pub(super) fn exact_unquote_text(raw: &str) -> String {
        let trimmed = raw.trim();

        if let Some(inner) = trimmed.strip_prefix('"').and_then(|text| text.strip_suffix('"')) {
            return inner.to_string();
        }

        if let Some(inner) = trimmed.strip_prefix('\'').and_then(|text| text.strip_suffix('\'')) {
            return inner.to_string();
        }

        trimmed.to_string()
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

    pub(super) fn consume_optional_semicolon(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) {
        self.skip_ignorable(tokens);
        if let Ok(token) = tokens.curr(false) {
            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::Semi)) {
                let _ = tokens.bump();
            }
        }
    }

    /// Parse a simple literal for testing
    pub fn parse_literal(&self, value: &str) -> Result<AstNode, Box<dyn Glitch>> {
        if value.starts_with('"') && value.ends_with('"') {
            let string_val = value[1..value.len() - 1].to_string();
            return Ok(AstNode::Literal(Literal::String(string_val)));
        }

        if value.starts_with('\'') && value.ends_with('\'') {
            let inner = &value[1..value.len() - 1];
            let mut chars = inner.chars();
            return match (chars.next(), chars.next()) {
                (Some(ch), None) => Ok(AstNode::Literal(Literal::Character(ch))),
                _ => Ok(AstNode::Literal(Literal::String(inner.to_string()))),
            };
        }

        let normalized = value.replace('_', "");

        if let Some(hex) = normalized
            .strip_prefix("0x")
            .or_else(|| normalized.strip_prefix("0X"))
        {
            if let Ok(int_val) = i64::from_str_radix(hex, 16) {
                return Ok(AstNode::Literal(Literal::Integer(int_val)));
            }
        }

        if let Some(octal) = normalized
            .strip_prefix("0o")
            .or_else(|| normalized.strip_prefix("0O"))
        {
            if let Ok(int_val) = i64::from_str_radix(octal, 8) {
                return Ok(AstNode::Literal(Literal::Integer(int_val)));
            }
        }

        if let Some(binary) = normalized
            .strip_prefix("0b")
            .or_else(|| normalized.strip_prefix("0B"))
        {
            if let Ok(int_val) = i64::from_str_radix(binary, 2) {
                return Ok(AstNode::Literal(Literal::Integer(int_val)));
            }
        }

        if let Ok(int_val) = normalized.parse::<i64>() {
            return Ok(AstNode::Literal(Literal::Integer(int_val)));
        }

        if let Ok(float_val) = normalized.parse::<f64>() {
            return Ok(AstNode::Literal(Literal::Float(float_val)));
        }

        // Default to identifier
        Ok(AstNode::Identifier {
            name: value.to_string(),
        })
    }
}
