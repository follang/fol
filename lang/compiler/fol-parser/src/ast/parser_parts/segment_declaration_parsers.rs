use super::*;

impl AstParser {
    pub(super) fn parse_seg_decl(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
    ) -> Result<AstNode, ParseError> {
        let seg_token = tokens.curr(false)?;
        if !matches!(seg_token.key(), KEYWORD::Keyword(BUILDIN::Seg)) {
            return Err(ParseError::from_token(
                &seg_token,
                "Expected 'seg' declaration".to_string(),
            ));
        }

        let _ = tokens.bump();
        self.skip_ignorable(tokens)?;
        let options = self.parse_decl_visibility_options(tokens, "segment")?;
        self.skip_ignorable(tokens)?;

        let name_token = tokens.curr(false)?;
        let name = Self::expect_named_label(&name_token, "Expected segment name after 'seg'")?;
        let _ = tokens.bump();

        self.skip_ignorable(tokens)?;
        let colon = tokens.curr(false)?;
        if !matches!(colon.key(), KEYWORD::Symbol(SYMBOL::Colon)) {
            return Err(ParseError::from_token(
                &colon,
                "Expected ':' after segment name".to_string(),
            ));
        }
        let _ = tokens.bump();
        self.skip_ignorable(tokens)?;

        let def_type = self.parse_type_reference_tokens(tokens)?;
        if !matches!(def_type, FolType::Module { .. }) {
            return Err(ParseError::from_token(
                &seg_token,
                format!(
                    "Segment declarations require module types, found '{}'",
                    Self::fol_type_label(&def_type)
                ),
            ));
        }

        self.skip_ignorable(tokens)?;
        let assign = tokens.curr(false)?;
        if !matches!(assign.key(), KEYWORD::Symbol(SYMBOL::Equal)) {
            return Ok(AstNode::SegDecl {
                options,
                name,
                seg_type: def_type,
                body: Vec::new(),
            });
        }
        let _ = tokens.bump();

        self.skip_ignorable(tokens)?;
        let open = tokens.curr(false)?;
        if !matches!(open.key(), KEYWORD::Symbol(SYMBOL::CurlyO)) {
            return Err(ParseError::from_token(
                &open,
                "Expected '{' to start segment body".to_string(),
            ));
        }
        let _ = tokens.bump();

        let body = self.parse_block_body(tokens, "Expected '}' to close segment body")?;

        Ok(AstNode::SegDecl {
            options,
            name,
            seg_type: def_type,
            body,
        })
    }
}
