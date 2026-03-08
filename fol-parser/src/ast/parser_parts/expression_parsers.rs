use super::*;

impl AstParser {
    pub(super) fn parse_call_args(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut args = Vec::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                break;
            }

            args.push(self.parse_logical_expression(tokens)?);
            self.skip_ignorable(tokens);

            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                break;
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or ')' in call arguments".to_string(),
            )));
        }

        Ok(args)
    }

    pub(super) fn lookahead_is_assignment(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let mut found_compound_symbol = false;
        let mut square_depth = 0usize;
        let mut round_depth = 0usize;
        let mut expect_member_ident = false;
        let mut expect_path_ident = false;
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::SquarO)) {
                square_depth += 1;
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::SquarC)) {
                if square_depth == 0 {
                    return false;
                }
                square_depth -= 1;
                continue;
            }

            if square_depth > 0 {
                if matches!(key, KEYWORD::Symbol(SYMBOL::Equal))
                    || self.compound_assignment_op(&key).is_some()
                {
                    return true;
                }
                if matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                    round_depth += 1;
                } else if matches!(key, KEYWORD::Symbol(SYMBOL::RoundC)) && round_depth > 0 {
                    round_depth -= 1;
                }
                continue;
            }

            if round_depth > 0 {
                if matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                    round_depth += 1;
                } else if matches!(key, KEYWORD::Symbol(SYMBOL::RoundC)) {
                    round_depth -= 1;
                }
                continue;
            }

            if found_compound_symbol {
                return matches!(key, KEYWORD::Symbol(SYMBOL::Equal));
            }

            if expect_path_ident {
                if Self::token_can_be_logical_name(&key)
                    || matches!(key, KEYWORD::Literal(LITERAL::Stringy))
                {
                    expect_path_ident = false;
                    continue;
                }
                return false;
            }

            if expect_member_ident {
                if Self::token_can_be_logical_name(&key)
                    || matches!(key, KEYWORD::Literal(LITERAL::Stringy))
                {
                    expect_member_ident = false;
                    continue;
                }
                if matches!(key, KEYWORD::Symbol(SYMBOL::Equal))
                    || self.compound_assignment_op(&key).is_some()
                {
                    return true;
                }
                return false;
            }

            if matches!(key, KEYWORD::Operator(OPERATOR::Path)) {
                expect_path_ident = true;
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::Dot)) {
                expect_member_ident = true;
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                round_depth = 1;
                continue;
            }

            if self.compound_assignment_symbol_op(&key).is_some() {
                found_compound_symbol = true;
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::Equal))
                || self.compound_assignment_op(&key).is_some();
        }

        false
    }

    pub(super) fn lookahead_is_call(&self, tokens: &fol_lexer::lexer::stage3::Elements) -> bool {
        let mut allow_path_segment = false;
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if allow_path_segment {
                if Self::token_can_be_logical_name(&key)
                    || matches!(key, KEYWORD::Literal(LITERAL::Stringy))
                {
                    allow_path_segment = false;
                    continue;
                }
                return false;
            }

            if matches!(key, KEYWORD::Operator(OPERATOR::Path)) {
                allow_path_segment = true;
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::RoundO));
        }

        false
    }

    pub(super) fn lookahead_is_method_call(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let mut saw_dot = false;
        let mut allow_path_segment = false;
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if allow_path_segment {
                if Self::token_can_be_logical_name(&key)
                    || matches!(key, KEYWORD::Literal(LITERAL::Stringy))
                {
                    allow_path_segment = false;
                    continue;
                }
                return false;
            }

            if !saw_dot {
                if matches!(key, KEYWORD::Operator(OPERATOR::Path)) {
                    allow_path_segment = true;
                    continue;
                }
                if matches!(key, KEYWORD::Symbol(SYMBOL::Dot)) {
                    saw_dot = true;
                    continue;
                }
                return false;
            }

            if Self::token_can_be_logical_name(&key)
                || matches!(key, KEYWORD::Literal(LITERAL::Stringy))
            {
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::RoundO));
        }

        false
    }

    pub(super) fn can_start_assignment(&self, tokens: &fol_lexer::lexer::stage3::Elements) -> bool {
        match self.previous_significant_key(tokens) {
            None => true,
            Some(KEYWORD::Symbol(SYMBOL::CurlyO)) => true,
            Some(KEYWORD::Symbol(SYMBOL::Semi)) => true,
            Some(key) if key.is_terminal() => true,
            _ => false,
        }
    }

    pub(super) fn previous_significant_key(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> Option<KEYWORD> {
        for candidate in tokens.prev_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            return Some(key);
        }

        None
    }

    pub(super) fn bump_if_no_progress(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        before: (usize, usize, String),
    ) {
        if let Ok(current) = tokens.curr(false) {
            let after = (
                current.loc().row(),
                current.loc().col(),
                current.con().to_string(),
            );
            if before == after {
                let _ = tokens.bump();
            }
        }
    }

    pub(super) fn token_is_word(
        &self,
        token: &fol_lexer::lexer::stage3::element::Element,
        word: &str,
    ) -> bool {
        token.con().trim() == word
            || matches!(
                (token.key(), word),
                (KEYWORD::Keyword(BUILDIN::And), "and")
                    | (KEYWORD::Keyword(BUILDIN::Or), "or")
                    | (KEYWORD::Keyword(BUILDIN::Xor), "xor")
                    | (KEYWORD::Keyword(BUILDIN::Nand), "nand")
                    | (KEYWORD::Keyword(BUILDIN::Nor), "nor")
                    | (KEYWORD::Keyword(BUILDIN::Not), "not")
            )
    }

    pub(super) fn compound_assignment_op(&self, key: &KEYWORD) -> Option<BinaryOperator> {
        match key {
            KEYWORD::Operator(OPERATOR::Addeq) => Some(BinaryOperator::Add),
            KEYWORD::Operator(OPERATOR::Subeq) => Some(BinaryOperator::Sub),
            KEYWORD::Operator(OPERATOR::Multeq) => Some(BinaryOperator::Mul),
            KEYWORD::Operator(OPERATOR::Diveq) => Some(BinaryOperator::Div),
            _ => None,
        }
    }

    pub(super) fn compound_assignment_symbol_op(&self, key: &KEYWORD) -> Option<BinaryOperator> {
        match key {
            KEYWORD::Symbol(SYMBOL::Percent) => Some(BinaryOperator::Mod),
            KEYWORD::Symbol(SYMBOL::Carret) => Some(BinaryOperator::Pow),
            _ => None,
        }
    }

    pub(super) fn parse_logical_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.parse_pipe_expression(tokens)
    }

    pub(super) fn parse_logical_or_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_logical_xor_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            if self.token_is_word(&op_token, "or") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_logical_xor_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op: BinaryOperator::Or,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            if self.token_is_word(&op_token, "nor") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_logical_xor_expression(tokens)?;
                lhs = AstNode::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(AstNode::BinaryOp {
                        op: BinaryOperator::Or,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub(super) fn parse_logical_xor_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_logical_and_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            if self.token_is_word(&op_token, "xor") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_logical_and_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op: BinaryOperator::Xor,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub(super) fn parse_logical_and_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_comparison_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            if self.token_is_word(&op_token, "and") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_comparison_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op: BinaryOperator::And,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            if self.token_is_word(&op_token, "nand") {
                self.consume_significant_token(tokens);
                let rhs = self.parse_comparison_expression(tokens)?;
                lhs = AstNode::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(AstNode::BinaryOp {
                        op: BinaryOperator::And,
                        left: Box::new(lhs),
                        right: Box::new(rhs),
                    }),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub(super) fn parse_comparison_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_range_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            let next_key = self.next_significant_key_from_window(tokens);
            let op_text = op_token.con().trim().to_string();
            let (binary_op, consume_count) = match op_token.key() {
                KEYWORD::Keyword(BUILDIN::Cast) => (Some(BinaryOperator::Cast), 1),
                KEYWORD::Keyword(BUILDIN::As) => (Some(BinaryOperator::As), 1),
                KEYWORD::Keyword(BUILDIN::Is) => (Some(BinaryOperator::Is), 1),
                KEYWORD::Keyword(BUILDIN::Has) => (Some(BinaryOperator::Has), 1),
                KEYWORD::Keyword(BUILDIN::In) => (Some(BinaryOperator::In), 1),
                KEYWORD::Operator(OPERATOR::Equal) => (Some(BinaryOperator::Eq), 1),
                KEYWORD::Operator(OPERATOR::Noteq) => (Some(BinaryOperator::Ne), 1),
                KEYWORD::Operator(OPERATOR::Greateq) => (Some(BinaryOperator::Ge), 1),
                KEYWORD::Operator(OPERATOR::Lesseq) => (Some(BinaryOperator::Le), 1),
                KEYWORD::Symbol(SYMBOL::AngleC) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Ge), 2)
                    } else {
                        (Some(BinaryOperator::Gt), 1)
                    }
                }
                KEYWORD::Symbol(SYMBOL::AngleO) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Le), 2)
                    } else {
                        (Some(BinaryOperator::Lt), 1)
                    }
                }
                KEYWORD::Symbol(SYMBOL::Equal) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Eq), 2)
                    } else {
                        (None, 0)
                    }
                }
                KEYWORD::Symbol(SYMBOL::Bang) => {
                    if matches!(next_key, Some(KEYWORD::Symbol(SYMBOL::Equal))) {
                        (Some(BinaryOperator::Ne), 2)
                    } else {
                        (None, 0)
                    }
                }
                _ => match op_text.as_str() {
                    ">=" => (Some(BinaryOperator::Ge), 1),
                    "<=" => (Some(BinaryOperator::Le), 1),
                    "==" => (Some(BinaryOperator::Eq), 1),
                    "!=" => (Some(BinaryOperator::Ne), 1),
                    ">" => (Some(BinaryOperator::Gt), 1),
                    "<" => (Some(BinaryOperator::Lt), 1),
                    _ => (None, 0),
                },
            };

            if let Some(op) = binary_op {
                for _ in 0..consume_count {
                    self.consume_significant_token(tokens);
                }
                let rhs = self.parse_range_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub(super) fn parse_range_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        if let Ok(token) = tokens.curr(false) {
            let is_open_start_range = matches!(
                token.key(),
                KEYWORD::Operator(OPERATOR::Dotdot) | KEYWORD::Operator(OPERATOR::Dotdotdot)
            ) || matches!(token.con().trim(), ".." | "...");
            let inclusive = !matches!(token.key(), KEYWORD::Operator(OPERATOR::Dotdotdot))
                && token.con().trim() != "...";
            if is_open_start_range {
                let operator_token = token.clone();
                let _ = tokens.bump();
                self.skip_ignorable(tokens);

                let next = tokens.curr(false)?;
                if next.key().is_terminal()
                    || matches!(
                        next.key(),
                        KEYWORD::Symbol(SYMBOL::Comma)
                            | KEYWORD::Symbol(SYMBOL::RoundC)
                            | KEYWORD::Symbol(SYMBOL::CurlyC)
                            | KEYWORD::Symbol(SYMBOL::SquarC)
                    )
                {
                    return Err(Box::new(ParseError::from_token(
                        &operator_token,
                        "Expected expression after '..'".to_string(),
                    )));
                }

                let rhs = self.parse_add_sub_expression(tokens)?;
                return Ok(AstNode::Range {
                    start: None,
                    end: Some(Box::new(rhs)),
                    inclusive,
                });
            }
        }

        let lhs = self.parse_add_sub_expression(tokens)?;
        self.skip_ignorable(tokens);

        let op_token = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(lhs),
        };

        let is_range = matches!(
            op_token.key(),
            KEYWORD::Operator(OPERATOR::Dotdot) | KEYWORD::Operator(OPERATOR::Dotdotdot)
        ) || matches!(op_token.con().trim(), ".." | "...");
        if !is_range {
            return Ok(lhs);
        }
        let inclusive = !matches!(op_token.key(), KEYWORD::Operator(OPERATOR::Dotdotdot))
            && op_token.con().trim() != "...";

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let next = tokens.curr(false)?;
        if matches!(
            next.key(),
            KEYWORD::Symbol(SYMBOL::Comma)
                | KEYWORD::Symbol(SYMBOL::RoundC)
                | KEYWORD::Symbol(SYMBOL::CurlyC)
                | KEYWORD::Symbol(SYMBOL::SquarC)
        ) {
            return Ok(AstNode::Range {
                start: Some(Box::new(lhs)),
                end: None,
                inclusive,
            });
        }
        if next.key().is_terminal() {
            return Err(Box::new(ParseError::from_token(
                &op_token,
                format!("Expected expression after '{}'", op_token.con().trim()),
            )));
        }

        let rhs = self.parse_add_sub_expression(tokens)?;
        Ok(AstNode::Range {
            start: Some(Box::new(lhs)),
            end: Some(Box::new(rhs)),
            inclusive,
        })
    }

    pub(super) fn next_significant_key_from_window(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> Option<KEYWORD> {
        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            return Some(key);
        }

        None
    }

    pub(super) fn consume_significant_token(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) {
        for _ in 0..16 {
            if tokens.bump().is_none() {
                break;
            }

            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => break,
            };

            if !(token.key().is_void() || token.key().is_comment()) {
                break;
            }
        }
    }

    pub(super) fn parse_add_sub_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_mul_div_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            let binary_op = match op_token.key() {
                KEYWORD::Operator(OPERATOR::Add) | KEYWORD::Symbol(SYMBOL::Plus) => {
                    Some(BinaryOperator::Add)
                }
                KEYWORD::Operator(OPERATOR::Abstract) | KEYWORD::Symbol(SYMBOL::Minus) => {
                    Some(BinaryOperator::Sub)
                }
                _ => None,
            };

            if let Some(op) = binary_op {
                let _ = tokens.bump();
                let rhs = self.parse_mul_div_expression(tokens)?;

                lhs = AstNode::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub(super) fn parse_mul_div_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let mut lhs = self.parse_pow_expression(tokens)?;

        for _ in 0..32 {
            self.skip_ignorable(tokens);

            let op_token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(lhs),
            };

            let binary_op = match op_token.key() {
                KEYWORD::Operator(OPERATOR::Multiply) | KEYWORD::Symbol(SYMBOL::Star) => {
                    Some(BinaryOperator::Mul)
                }
                KEYWORD::Operator(OPERATOR::Divide) | KEYWORD::Symbol(SYMBOL::Root) => {
                    Some(BinaryOperator::Div)
                }
                KEYWORD::Symbol(SYMBOL::Percent) => Some(BinaryOperator::Mod),
                _ => None,
            };

            if let Some(op) = binary_op {
                let _ = tokens.bump();
                let rhs = self.parse_pow_expression(tokens)?;
                lhs = AstNode::BinaryOp {
                    op,
                    left: Box::new(lhs),
                    right: Box::new(rhs),
                };
                continue;
            }

            break;
        }

        Ok(lhs)
    }

    pub(super) fn parse_pow_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let lhs = self.parse_primary_expression(tokens)?;
        self.skip_ignorable(tokens);

        let op_token = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(lhs),
        };

        if !matches!(op_token.key(), KEYWORD::Symbol(SYMBOL::Carret)) {
            return Ok(lhs);
        }

        let _ = tokens.bump();
        let rhs = self.parse_pow_expression(tokens)?;
        Ok(AstNode::BinaryOp {
            op: BinaryOperator::Pow,
            left: Box::new(lhs),
            right: Box::new(rhs),
        })
    }

    pub(super) fn parse_primary_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        self.skip_ignorable(tokens);
        let token = tokens.curr(false)?;

        if let Some((message, unary_op)) = self.unary_prefix_info(&token) {
            let operator_token = token.clone();
            let _ = tokens.bump();
            self.ensure_unary_operand(tokens, &operator_token, message)?;

            let operand = self.parse_primary_expression(tokens)?;
            if let Some(op) = unary_op {
                return Ok(AstNode::UnaryOp {
                    op,
                    operand: Box::new(operand),
                });
            }

            return Ok(operand);
        }

        if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return self.parse_container_expression(tokens);
        }

        let node = if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            let _ = tokens.bump();
            let inner = self.parse_logical_expression(tokens)?;
            self.skip_ignorable(tokens);

            let close = tokens.curr(false)?;
            if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                return Err(Box::new(ParseError::from_token(
                    &close,
                    "Expected closing ')' for parenthesized expression".to_string(),
                )));
            }

            let _ = tokens.bump();
            inner
        } else if matches!(token.key(), KEYWORD::Literal(LITERAL::Stringy))
            && matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Symbol(SYMBOL::RoundO))
            )
        {
            let name = Self::token_to_named_label(&token).ok_or_else(|| {
                Box::new(ParseError::from_token(
                    &token,
                    "Expected quoted callable name".to_string(),
                )) as Box<dyn Glitch>
            })?;
            let _ = tokens.bump();
            AstNode::Identifier { name }
        } else if Self::token_can_start_path_expression(&token)
            && matches!(
                self.next_significant_key_from_window(tokens),
                Some(KEYWORD::Operator(OPERATOR::Path))
            )
        {
            let name = self.parse_named_path(
                tokens,
                "Expected expression path root",
                "Expected name after '::' in expression path",
            )?;
            AstNode::Identifier { name }
        } else {
            let node = self.parse_primary(&token)?;
            let _ = tokens.bump();
            node
        };

        self.parse_postfix_expression(tokens, node)
    }

    pub(super) fn parse_container_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start container expression".to_string(),
            )));
        }
        let _ = tokens.bump();

        let mut elements = Vec::new();
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            let expr = self.parse_logical_expression(tokens)?;
            self.skip_ignorable(tokens);

            if let Ok(next) = tokens.curr(false) {
                if matches!(next.key(), KEYWORD::Keyword(BUILDIN::For)) {
                    if !elements.is_empty() {
                        return Err(Box::new(ParseError::from_token(
                            &next,
                            "Rolling expressions must contain exactly one output expression"
                                .to_string(),
                        )));
                    }
                    return self.parse_rolling_expression(tokens, expr);
                }
            }

            elements.push(expr);

            let sep = tokens.curr(false)?;
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::Comma)) {
                let _ = tokens.bump();
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) {
                let _ = tokens.bump();
                break;
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',' or '}' in container expression".to_string(),
            )));
        }

        if elements.len() == 1 {
            if let Some(range) = elements.pop() {
                if matches!(range, AstNode::Range { .. }) {
                    return Ok(range);
                }
                return Ok(AstNode::ContainerLiteral {
                    container_type: ContainerType::Array,
                    elements: vec![range],
                });
            }
        }

        Ok(AstNode::ContainerLiteral {
            container_type: ContainerType::Array,
            elements,
        })
    }

    pub(super) fn parse_postfix_expression(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        mut node: AstNode,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        for _ in 0..256 {
            self.skip_ignorable(tokens);

            let token = match tokens.curr(false) {
                Ok(token) => token,
                Err(_) => return Ok(node),
            };

            match token.key() {
                KEYWORD::Symbol(SYMBOL::RoundO) => {
                    let name = match &node {
                        AstNode::Identifier { name } => name.clone(),
                        _ => break,
                    };
                    let _ = tokens.bump();
                    let args = self.parse_call_args(tokens)?;
                    node = AstNode::FunctionCall { name, args };
                }
                KEYWORD::Symbol(SYMBOL::Dot) => {
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);

                    let member_token = tokens.curr(false)?;
                    let member = Self::token_to_named_label(&member_token).ok_or_else(|| {
                        Box::new(ParseError::from_token(
                            &member_token,
                            "Expected field or method name after '.'".to_string(),
                        )) as Box<dyn Glitch>
                    })?;
                    let _ = tokens.bump();
                    self.skip_ignorable(tokens);

                    let is_method_call = matches!(
                        tokens.curr(false).map(|token| token.key()),
                        Ok(KEYWORD::Symbol(SYMBOL::RoundO))
                    );

                    if is_method_call {
                        let _ = tokens.bump();
                        let args = self.parse_call_args(tokens)?;
                        node = AstNode::MethodCall {
                            object: Box::new(node),
                            method: member,
                            args,
                        };
                    } else {
                        node = AstNode::FieldAccess {
                            object: Box::new(node),
                            field: member,
                        };
                    }
                }
                KEYWORD::Symbol(SYMBOL::SquarO) => {
                    node = self.parse_index_or_slice_expression(tokens, node)?;
                }
                _ => break,
            }
        }

        Ok(node)
    }
}
