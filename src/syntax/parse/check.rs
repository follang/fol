use colored::Colorize;
use crate::types::*;
use crate::syntax::lexer;
use crate::syntax::token::*;
use crate::syntax::point;
use crate::syntax::index;
use crate::syntax::parse::{eater, Parse};

pub fn expect(lex: &mut lexer::Elements, keyword: KEYWORD, ignore: bool) -> Vod {
    if lex.curr(ignore)?.key() == keyword { return Ok(()) };
    let wrong = lex.curr(ignore)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: keyword,
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_many(lex: &mut lexer::Elements, keywords: Vec<KEYWORD>, ignore: bool) -> Vod {
    let wrong = lex.curr(ignore)?;
    if let Some(_) = keywords.iter().find(|&x| x == &wrong.key()) { return Ok(()) }
    Err( catch!( Typo::ParserManyUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        keys: keywords,
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_ident(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Identifier) { return Ok(()) };
    let wrong = lex.curr(ignore)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Identifier, 
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_ident_literal(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Identifier) || matches!(lex.curr(ignore)?.key(), KEYWORD::Literal(_)) { return Ok(()) };
    let wrong = lex.curr(ignore)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Identifier, 
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_literal(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Literal(_)) { return Ok(()) };
    let wrong = lex.curr(ignore)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Literal(LITERAL::ANY), 
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_buildin(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Keyword(_)) { return Ok(()) };
    let wrong = lex.curr(ignore)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Keyword(BUILDIN::ANY), 
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_symbol(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Symbol(_)) { return Ok(()) };
    let wrong = lex.curr(ignore)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Symbol(SYMBOL::ANY), 
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_operator(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Operator(_)) { return Ok(()) };
    let wrong = lex.curr(ignore)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Operator(OPERATOR::ANY), 
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_void(lex: &mut lexer::Elements) -> Vod {
    if matches!(lex.curr(false)?.key(), KEYWORD::Void(_)) { return Ok(()) };
    let wrong = lex.curr(false)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Void(VOID::ANY), 
        src: wrong.loc().source().clone()
    }))
}
pub fn expect_terminal(lex: &mut lexer::Elements) -> Vod {
    if lex.curr(false)?.key().is_terminal() { return Ok(()) };
    let wrong = lex.curr(false)?;
    eater::until_term(lex, true)?;
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(wrong.loc().clone()), 
        key1: wrong.key(), 
        key2: KEYWORD::Void(VOID::ANY), 
        src: wrong.loc().source().clone()
    }))
}
pub fn needs_body(pt: point::Location, lex: &mut lexer::Elements, body: &dyn Parse) -> Vod {
    let wrong = lex.curr(false)?;
    if body.nodes().len() > 0 { return Ok(()) };
    Err( catch!( Typo::ParserNeedsBody{ 
        msg: None,
        loc: Some(pt), 
        src: wrong.loc().source().clone()
    }))
}
pub fn type_balance(idents: usize, dt: usize, loc: &point::Location, src: &Option<index::Source>) -> Vod {
    if dt > idents {
        return Err( catch!( Typo::ParserTypeDisbalance {
            msg: Some(format!(
                "number of identifiers: {} is smaller than number of types: {}",
                format!("[ {} ]", idents).black().on_red(), format!("[ {} ]", dt).black().on_red(),
                )),
            loc: Some(loc.clone()), 
            src: src.clone(),
        }))
    }
    Ok(())
}

pub fn unexpected_top(lex: &mut lexer::Elements, el: lexer::Element) -> Vod {
    lex.until_term(false)?;
    Err( catch!( Typo::ParserTopForbid{ 
        loc: Some(el.loc().clone()), 
        key1: el.key().clone(), 
        keys: vec![
            KEYWORD::Keyword(BUILDIN::Use),
            KEYWORD::Keyword(BUILDIN::Def),
            KEYWORD::Keyword(BUILDIN::Fun),
            KEYWORD::Keyword(BUILDIN::Pro),
            KEYWORD::Keyword(BUILDIN::Log),
            KEYWORD::Keyword(BUILDIN::Typ),
            KEYWORD::Keyword(BUILDIN::Ali),
            KEYWORD::Keyword(BUILDIN::Imp),
            KEYWORD::Keyword(BUILDIN::Con),
        ],
        src: el.loc().clone().source()
    }))
}

pub fn unexpected_imp(lex: &mut lexer::Elements, el: lexer::Element) -> Vod {
    lex.until_term(false)?;
    Err( catch!( Typo::ParserImpForbid{ 
        loc: Some(el.loc().clone()), 
        key1: el.key().clone(), 
        keys: vec![
            KEYWORD::Keyword(BUILDIN::Fun),
            KEYWORD::Keyword(BUILDIN::Pro),
            KEYWORD::Keyword(BUILDIN::Log),
        ],
        src: el.loc().clone().source()
    }))
}

pub fn unexpected_typ(lex: &mut lexer::Elements, el: lexer::Element) -> Vod {
    lex.until_term(false)?;
    Err( catch!( Typo::ParserTypForbid{ 
        loc: Some(el.loc().clone()), 
        key1: el.key().clone(), 
        keys: vec![
            KEYWORD::Keyword(BUILDIN::Var),
            KEYWORD::Keyword(BUILDIN::Fun),
            KEYWORD::Keyword(BUILDIN::Pro),
            KEYWORD::Keyword(BUILDIN::Log),
            KEYWORD::Keyword(BUILDIN::Lab),
        ],
        src: el.loc().clone().source()
    }))
}

pub fn unexpected_fun(lex: &mut lexer::Elements, el: lexer::Element) -> Vod {
    lex.until_term(false)?;
    Err( catch!( Typo::ParserFunForbid{ 
        loc: Some(el.loc().clone()), 
        key1: el.key().clone(), 
        keys: vec![
            KEYWORD::Keyword(BUILDIN::Var),
            KEYWORD::Keyword(BUILDIN::Fun),
            KEYWORD::Keyword(BUILDIN::Pro),
            KEYWORD::Keyword(BUILDIN::Log),
            KEYWORD::Keyword(BUILDIN::Lab),
        ],
        src: el.loc().clone().source()
    }))
}
