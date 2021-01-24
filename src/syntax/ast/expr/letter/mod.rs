use crate::syntax::ast::Ast;

#[derive(Clone, Debug)]
pub enum LetterExpr {
    string_normal,
    string_raw,
    string_formated,
    char_normal(char),
    char_binary(u8),
}

impl Ast for LetterExpr {}
