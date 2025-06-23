use fol_types::*;
use fol_lexer;
use crate::token::*;

pub fn body_top(lex: &mut lexer::Elements, ignore: bool) -> Con<bool> {
    let key = if matches!(lex.curr(ignore)?.key(), KEYWORD::Symbol(_)) { lex.peek(0, ignore)?.key() } else { lex.curr(ignore)?.key() };
    match key {
        KEYWORD::Keyword(BUILDIN::Seg) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Use) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Def) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Fun) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Pro) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Typ) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Ali) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Imp) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Con) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Var) => Ok(true),
        _ => Ok(false),
    }
}
pub fn body_imp(lex: &mut lexer::Elements, ignore: bool) -> Con<bool> {
    let key = if matches!(lex.curr(ignore)?.key(), KEYWORD::Symbol(_)) { lex.peek(0, ignore)?.key() } else { lex.curr(ignore)?.key() };
    match key {
        KEYWORD::Keyword(BUILDIN::Fun) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Pro) => Ok(true),
        _ => Ok(false),
    }
}
pub fn body_typ(lex: &mut lexer::Elements, ignore: bool) -> Con<bool> {
    let key = if matches!(lex.curr(ignore)?.key(), KEYWORD::Symbol(_)) { lex.peek(0, ignore)?.key() } else { lex.curr(ignore)?.key() };
    match key {
        KEYWORD::Keyword(BUILDIN::Var) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Fun) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Pro) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Lab) => Ok(true),
        _ => Ok(false),
    }
}
pub fn body_fun(lex: &mut lexer::Elements, ignore: bool) -> Con<bool> {
    let key = if matches!(lex.curr(ignore)?.key(), KEYWORD::Symbol(_)) { lex.peek(0, ignore)?.key() } else { lex.curr(ignore)?.key() };
    match key {
        KEYWORD::Keyword(BUILDIN::Seg) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Var) => Ok(true),
        KEYWORD::Keyword(BUILDIN::Lab) => Ok(true),
        _ => Ok(false),
    }
}
