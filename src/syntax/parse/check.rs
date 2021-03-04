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
    if let Some(e) = keywords.iter().find(|&x| x == currkey) {
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
    if matches!(lex.curr(ignore)?.key(), KEYWORD::ident) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::ident, 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_ident_literal(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::ident) || matches!(lex.curr(ignore)?.key(), KEYWORD::literal(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::ident, 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_literal(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::literal(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::literal(LITERAL::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_buildin(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::buildin(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::buildin(BUILDIN::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_symbol(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::symbol(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::symbol(SYMBOL::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_operator(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::operator(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::operator(OPERATOR::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_void(lex: &mut lexer::Elements) -> Vod {
    if matches!(lex.curr(false)?.key(), KEYWORD::void(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(false)?.loc().clone()), 
        key1: lex.curr(false)?.key(), 
        key2: KEYWORD::void(VOID::ANY), 
        src: lex.curr(false)?.loc().source().clone()
    }))
}
pub fn expect_terminal(lex: &mut lexer::Elements) -> Vod {
    if lex.curr(false)?.key().is_terminal() { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(false)?.loc().clone()), 
        key1: lex.curr(false)?.key(), 
        key2: KEYWORD::void(VOID::ANY), 
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
