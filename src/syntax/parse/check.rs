use colored::Colorize;
use crate::types::*;
use crate::syntax::lexer;
use crate::syntax::token::*;
use crate::syntax::point;
use crate::syntax::index;

pub fn expect(lex: &mut lexer::Elements, keyword: KEYWORD, ignore: bool) -> Vod {
    if lex.curr(ignore)?.key() == keyword {
        return Ok(())
    };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: keyword,
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_many(lex: &mut lexer::Elements, keywords: Vec<KEYWORD>, ignore: bool) -> Vod {
    let currkey = &lex.curr(ignore)?.key();
    if let Some(_) = keywords.iter().find(|&x| x == currkey) {
        return Ok(())
    }
    Err( catch!( Typo::ParserManyUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        keys: keywords,
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_ident(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Identifier) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::Identifier, 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_ident_literal(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Identifier) || matches!(lex.curr(ignore)?.key(), KEYWORD::Literal(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::Identifier, 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_literal(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Literal(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::Literal(LITERAL::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_buildin(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Keyword(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::Keyword(BUILDIN::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_symbol(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Symbol(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::Symbol(SYMBOL::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_operator(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::Operator(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::Operator(OPERATOR::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_void(lex: &mut lexer::Elements) -> Vod {
    if matches!(lex.curr(false)?.key(), KEYWORD::Void(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(false)?.loc().clone()), 
        key1: lex.curr(false)?.key(), 
        key2: KEYWORD::Void(VOID::ANY), 
        src: lex.curr(false)?.loc().source().clone()
    }))
}
pub fn expect_terminal(lex: &mut lexer::Elements) -> Vod {
    if lex.curr(false)?.key().is_terminal() { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(false)?.loc().clone()), 
        key1: lex.curr(false)?.key(), 
        key2: KEYWORD::Void(VOID::ANY), 
        src: lex.curr(false)?.loc().source().clone()
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
