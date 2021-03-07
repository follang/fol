use crate::types::Vod;
use crate::syntax::lexer;
use crate::syntax::token::*;
use crate::syntax::parse::check;

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

pub fn until_term(lex: &mut lexer::Elements, term: bool) -> Vod {
    while !lex.curr(true)?.key().is_eof() {
        if lex.curr(false)?.key().is_terminal() {
            if term { lex.bump(); }
            break
        }
        lex.bump();
    }
    Ok(())
}

pub fn stat_body(lex: &mut lexer::Elements, deep: isize) -> Vod {
    loop{
        if (matches!(lex.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) && lex.curr(false)?.loc().deep() == deep ) 
            || lex.curr(false)?.key().is_eof() {
            break
        }
        lex.jump(0, false)?;
    }
    lex.jump(0, false)?;
    Ok(())
}

pub fn expr_body2(lex: &mut lexer::Elements) -> Vod {
    let deep = lex.curr(false)?.loc().deep() - 1;
    loop{
        if (matches!(lex.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) && lex.curr(false)?.loc().deep() == deep ) 
            || lex.curr(false)?.key().is_eof() {
            break
        }
        lex.jump(0, false)?;
    }
    Ok(())
}


pub fn expr_body3(lex: &mut lexer::Elements) -> Vod {
    check::expect(lex, KEYWORD::Symbol(SYMBOL::CurlyO), true)?;
    let deep = lex.curr(false)?.loc().deep();
    loop{
        // println!("{}\t{}", lex.curr(false)?.loc(), lex.curr(false)?.key());
        if (matches!(lex.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) && lex.curr(false)?.loc().deep() == deep ) 
            || lex.curr(false)?.key().is_eof() {
            break
        }
        lex.jump(0, false)?;
    }
    lex.jump(0, false)?;
    Ok(())
}
