use super::*;

impl AstParser {
    pub(super) fn try_parse_source_kind_type_suffix(
        &self,
        tokens: &mut fol_lexer::lexer::stage3::Elements,
        base_name: &str,
    ) -> Result<Option<FolType>, Box<dyn Glitch>> {
        match base_name {
            "path" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() > 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected zero or one type argument for path[...]".to_string(),
                    )));
                }
                let name = match args.into_iter().next() {
                    None => String::new(),
                    Some(FolType::Named { name }) => name,
                    Some(other) => Self::fol_type_label(&other),
                };
                Ok(Some(FolType::Path { name }))
            }
            "url" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() > 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected zero or one type argument for url[...]".to_string(),
                    )));
                }
                let name = match args.into_iter().next() {
                    None => String::new(),
                    Some(FolType::Named { name }) => name,
                    Some(other) => Self::fol_type_label(&other),
                };
                Ok(Some(FolType::Url { name }))
            }
            "loc" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() > 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected zero or one type argument for loc[...]".to_string(),
                    )));
                }
                let name = match args.into_iter().next() {
                    None => String::new(),
                    Some(FolType::Named { name }) => name,
                    Some(other) => Self::fol_type_label(&other),
                };
                Ok(Some(FolType::Location { name }))
            }
            "std" => {
                let args = self.parse_type_argument_list(tokens)?;
                if args.len() > 1 {
                    let token = tokens.curr(false)?;
                    return Err(Box::new(ParseError::from_token(
                        &token,
                        "Expected zero or one type argument for std[...]".to_string(),
                    )));
                }
                let name = match args.into_iter().next() {
                    None => String::new(),
                    Some(FolType::Named { name }) => name,
                    Some(other) => Self::fol_type_label(&other),
                };
                Ok(Some(FolType::Standard { name }))
            }
            _ => Ok(None),
        }
    }

    pub(super) fn lower_bare_source_kind_type_name(name: &str) -> Option<FolType> {
        match name {
            "path" => Some(FolType::Path {
                name: String::new(),
            }),
            "url" => Some(FolType::Url {
                name: String::new(),
            }),
            "loc" => Some(FolType::Location {
                name: String::new(),
            }),
            "std" => Some(FolType::Standard {
                name: String::new(),
            }),
            _ => None,
        }
    }
}
