use super::*;

impl AstParser {
    pub(super) fn parse_imp_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, Box<dyn Glitch>> {
        let imp_token = tokens.curr(false)?;
        if !matches!(imp_token.key(), KEYWORD::Keyword(BUILDIN::Imp)) {
            return Err(Box::new(ParseError::from_token(
                &imp_token,
                "Expected 'imp' declaration".to_string(),
            )));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);
        self.parse_empty_imp_options(tokens)?;
        self.skip_ignorable(tokens);
        let generics = self.parse_type_generic_header(tokens)?;
        self.skip_ignorable(tokens);

        let name_token = tokens.curr(false)?;
        let name = Self::token_to_named_label(&name_token).ok_or_else(|| {
            Box::new(ParseError::from_token(
                &name_token,
                "Expected implementation name after 'imp'".to_string(),
            )) as Box<dyn Glitch>
        })?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(Box::new(ParseError::from_token(
                &colon,
                "Expected ':' after implementation name".to_string(),
            )));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let target = self.parse_type_reference_tokens(tokens)?;

        self.skip_ignorable(tokens);
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Err(Box::new(ParseError::from_token(
                &assign,
                "Expected '=' before implementation body".to_string(),
            )));
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens);
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(Box::new(ParseError::from_token(
                &open,
                "Expected '{' to start implementation body".to_string(),
            )));
        }
        let _ = tokens.bump();

        let body = self.parse_block_body(tokens, "Expected '}' to close implementation body")?;
        self.consume_optional_semicolon(tokens);

        Ok(AstNode::ImpDecl {
            generics,
            name,
            target,
            body,
        })
    }

    fn parse_empty_imp_options(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<(), Box<dyn Glitch>> {
        let open = match tokens.curr(false) {
            Ok(token) => token,
            Err(_) => return Ok(()),
        };

        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::SquarO)) {
            return Ok(());
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens);

        let close = tokens.curr(false)?;
        if !matches!(close.key(), KEYWORD::Symbol(SYMBOL::SquarC)) {
            return Err(Box::new(ParseError::from_token(
                &close,
                "Implementation options currently support only empty brackets".to_string(),
            )));
        }
        let _ = tokens.bump();
        Ok(())
    }
}
