use super::*;

impl AstParser {
    pub(super) fn lookahead_is_dot_builtin_call(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let current = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return false,
        };
        if !matches!(current.key(), KEYWORD::Symbol(SYMBOL::Dot)) {
            return false;
        }

        let mut saw_name = false;

        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if !saw_name {
                if Self::token_can_be_logical_name(&key) || key.is_textual_literal() || key.is_illegal() {
                    saw_name = true;
                    continue;
                }
                return false;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::RoundO));
        }

        false
    }

    pub(super) fn parse_dot_builtin_call_expr(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let dot = tokens.curr(false)?;
        if !matches!(dot.key(), KEYWORD::Symbol(SYMBOL::Dot)) {
            return Err(Box::new(ParseError::from_token(
                &dot,
                "Expected '.' to start builtin root call".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = Self::expect_named_label(&name_token, "Expected builtin call name after '.'")?;
        let _ = tokens.bump();

        let args =
            self.parse_open_paren_and_call_args(tokens, "Expected '(' after builtin call name")?;

        Ok(AstNode::FunctionCall { name, args })
    }

    fn lookahead_is_named_call_arg(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let current = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return false,
        };
        if !(Self::token_to_named_label(&current).is_some() || current.key().is_illegal()) {
            return false;
        }

        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };
            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }
            return matches!(key, KEYWORD::Symbol(SYMBOL::Equal));
        }

        false
    }

    fn parse_call_argument(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let current = tokens.curr(false)?;
        if current.con().trim() == "..." {
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let value_token = tokens.curr(false)?;
            if matches!(
                value_token.key(),
                KEYWORD::Symbol(SYMBOL::RoundC)
                    | KEYWORD::Symbol(SYMBOL::Comma)
                    | KEYWORD::Symbol(SYMBOL::Semi)
            ) {
                return Err(Box::new(ParseError::from_token(
                    &value_token,
                    "Expected expression after '...' in call arguments".to_string(),
                )));
            }

            let value = self.parse_logical_expression(tokens)?;
            return Ok(AstNode::Unpack {
                value: Box::new(value),
            });
        }

        if self.lookahead_is_named_call_arg(tokens) {
            let name_token = tokens.curr(false)?;
            let name =
                Self::expect_named_label(&name_token, "Expected argument name before '='")?;
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let equal = tokens.curr(false)?;
            if !matches!(equal.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
                return Err(Box::new(ParseError::from_token(
                    &equal,
                    "Expected '=' after named call argument".to_string(),
                )));
            }
            let _ = tokens.bump();
            self.skip_ignorable(tokens);

            let value = self.parse_logical_expression(tokens)?;
            return Ok(AstNode::NamedArgument {
                name,
                value: Box::new(value),
            });
        }

        self.parse_logical_expression(tokens)
    }

    pub(super) fn parse_call_args(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<Vec<AstNode>, Box<dyn Glitch>> {
        let mut args = Vec::new();
        let mut seen_named_arg = false;
        for _ in 0..256 {
            self.skip_ignorable(tokens);
            let token = tokens.curr(false)?;

            if matches!(token.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                break;
            }

            let arg = self.parse_call_argument(tokens)?;
            let is_named = matches!(arg, AstNode::NamedArgument { .. });
            if !is_named && seen_named_arg {
                return Err(Box::new(ParseError::from_token(
                    &token,
                    "Positional call arguments are not allowed after named arguments"
                        .to_string(),
                )));
            }
            seen_named_arg |= is_named;
            args.push(arg);
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
                    Ok(KEYWORD::Symbol(SYMBOL::RoundC))
                ) {
                    let _ = tokens.bump();
                    break;
                }
                continue;
            }
            if matches!(sep.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
                let _ = tokens.bump();
                break;
            }

            return Err(Box::new(ParseError::from_token(
                &sep,
                "Expected ',', ';', or ')' in call arguments".to_string(),
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
                if key.is_illegal() {
                    return true;
                }
                if Self::token_can_be_logical_name(&key) || key.is_textual_literal() {
                    expect_path_ident = false;
                    continue;
                }
                return false;
            }

            if expect_member_ident {
                if key.is_illegal() {
                    return true;
                }
                if Self::token_can_be_logical_name(&key) || key.is_textual_literal() {
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
                if Self::token_can_be_logical_name(&key) || key.is_textual_literal() {
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
                if Self::token_can_be_logical_name(&key) || key.is_textual_literal() {
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

            if key.is_illegal() {
                return true;
            }

            if Self::token_can_be_logical_name(&key) || key.is_textual_literal() {
                continue;
            }

            return matches!(key, KEYWORD::Symbol(SYMBOL::RoundO));
        }

        false
    }

    pub(super) fn lookahead_is_general_invoke(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
        starts_grouped: bool,
    ) -> bool {
        let mut round_depth = if starts_grouped { 1usize } else { 0usize };
        let mut square_depth = 0usize;
        let mut saw_postfix_base = starts_grouped;
        let mut expect_path_segment = false;
        let mut expect_member_name = false;

        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            if round_depth > 0 {
                if matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                    round_depth += 1;
                } else if matches!(key, KEYWORD::Symbol(SYMBOL::RoundC)) {
                    round_depth -= 1;
                    if round_depth == 0 {
                        saw_postfix_base = true;
                    }
                } else if matches!(key, KEYWORD::Symbol(SYMBOL::SquarO)) {
                    square_depth = 1;
                }
                continue;
            }

            if square_depth > 0 {
                if matches!(key, KEYWORD::Symbol(SYMBOL::SquarO)) {
                    square_depth += 1;
                } else if matches!(key, KEYWORD::Symbol(SYMBOL::SquarC)) {
                    square_depth -= 1;
                    if square_depth == 0 {
                        saw_postfix_base = true;
                    }
                } else if matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                    round_depth = 1;
                }
                continue;
            }

            if expect_path_segment {
                if Self::token_can_be_logical_name(&key) || key.is_textual_literal() {
                    expect_path_segment = false;
                    continue;
                }
                return false;
            }

            if expect_member_name {
                if Self::token_can_be_logical_name(&key) || key.is_textual_literal() {
                    expect_member_name = false;
                    continue;
                }
                return false;
            }

            if matches!(key, KEYWORD::Operator(OPERATOR::Path)) {
                expect_path_segment = true;
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::Dot)) {
                expect_member_name = true;
                saw_postfix_base = true;
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::Colon)) {
                saw_postfix_base = true;
                continue;
            }

            if matches!(key, KEYWORD::Symbol(SYMBOL::SquarO)) {
                square_depth = 1;
                saw_postfix_base = true;
                continue;
            }

            if saw_postfix_base && matches!(key, KEYWORD::Symbol(SYMBOL::RoundO)) {
                return true;
            }

            return false;
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

    pub(super) fn lookahead_has_top_level_pipe(
        &self,
        tokens: &fol_lexer::lexer::stage3::Elements,
    ) -> bool {
        let mut round_depth = 0usize;
        let mut square_depth = 0usize;
        let mut curly_depth = 0usize;

        for candidate in tokens.next_vec() {
            let token = match candidate {
                Ok(token) => token,
                Err(_) => continue,
            };

            let key = token.key();
            if key.is_void() || key.is_comment() {
                continue;
            }

            match key {
                KEYWORD::Symbol(SYMBOL::RoundO) => round_depth += 1,
                KEYWORD::Symbol(SYMBOL::RoundC) => {
                    if round_depth > 0 {
                        round_depth -= 1;
                    } else {
                        break;
                    }
                }
                KEYWORD::Symbol(SYMBOL::SquarO) => square_depth += 1,
                KEYWORD::Symbol(SYMBOL::SquarC) => {
                    square_depth = square_depth.saturating_sub(1);
                }
                KEYWORD::Symbol(SYMBOL::CurlyO) => curly_depth += 1,
                KEYWORD::Symbol(SYMBOL::CurlyC) => {
                    if curly_depth == 0 {
                        break;
                    }
                    curly_depth -= 1;
                }
                KEYWORD::Symbol(SYMBOL::Pipe)
                    if round_depth == 0 && square_depth == 0 && curly_depth == 0 =>
                {
                    return true;
                }
                _ if round_depth == 0
                    && square_depth == 0
                    && curly_depth == 0
                    && key.is_terminal() =>
                {
                    break;
                }
                _ => {}
            }
        }

        false
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

}
