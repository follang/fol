use super::*;

impl AstParser {
    pub(super) fn parse_imp_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, ParseError> {
        let imp_token = tokens.curr(false)?;
        if !matches!(imp_token.key(), KEYWORD::Keyword(BUILDIN::Imp)) {
            return Err(ParseError::from_token(
                &imp_token,
                "Expected 'imp' declaration".to_string(),
            ));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens)?;
        let options = self.parse_decl_visibility_options(tokens, "implementation")?;
        self.skip_ignorable(tokens)?;
        let generics = self.parse_type_generic_header(tokens)?;
        self.skip_ignorable(tokens)?;

        let name_token = tokens.curr(false)?;
        let name =
            Self::expect_named_label(&name_token, "Expected implementation name after 'imp'")?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens)?;
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(ParseError::from_token(
                &colon,
                "Expected ':' after implementation name".to_string(),
            ));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens)?;

        let target = self.parse_type_reference_tokens(tokens)?;

        self.skip_ignorable(tokens)?;
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Ok(AstNode::ImpDecl {
                options,
                generics,
                name,
                target,
                body: Vec::new(),
            });
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens)?;
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(ParseError::from_token(
                &open,
                "Expected '{' to start implementation body".to_string(),
            ));
        }
        let _ = tokens.bump();

        let body = self.parse_block_body(tokens, "Expected '}' to close implementation body")?;

        Ok(AstNode::ImpDecl {
            options,
            generics,
            name,
            target,
            body,
        })
    }
}
