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
pub fn expect_option(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::option(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::option(OPTION::ANY),
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_assign(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::assign(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::assign(ASSIGN::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_types(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::types(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::types(TYPE::ANY), 
        src: lex.curr(ignore)?.loc().source().clone()
    }))
}
pub fn expect_form(lex: &mut lexer::Elements, ignore: bool) -> Vod {
    if matches!(lex.curr(ignore)?.key(), KEYWORD::form(_)) { return Ok(()) };
    Err( catch!( Typo::ParserUnexpected{ 
        loc: Some(lex.curr(ignore)?.loc().clone()), 
        key1: lex.curr(ignore)?.key(), 
        key2: KEYWORD::form(FORM::ANY), 
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
pub fn until_key(lex: &mut lexer::Elements, keywords: Vec<KEYWORD>) -> Vod {
    loop{ 
        if keywords.iter().any(|i| *i == lex.curr(false).unwrap_or(lex.default()).key()) { 
            break
        }
        lex.jump(0, false)?;
    }
    Ok(())
}

pub fn until_bracket(lex: &mut lexer::Elements) -> Vod {
    let deep = lex.curr(false)?.loc().deep() - 1;
    loop{
        if (lex.curr(false)?.key().is_close_bracket() && lex.curr(false)?.loc().deep() == deep) 
            || lex.curr(false)?.key().is_eof() {
            break
        }
        lex.jump(0, false)?;
    }
    lex.jump(0, false)?;
    Ok(())
}

pub fn type_balance(idents: usize, dt: usize, loc: &point::Location, src: &index::Source) -> Vod {
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
