use crate::types::{Vod, Errors};

use crate::syntax::nodes::Nodes;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, branch, Body, eater};

pub mod parameters;
pub mod generics;
pub mod datatype;
pub mod assign;
pub mod ident;
pub use crate::syntax::parse::stat::{
    parameters::*,
    generics::*,
    datatype::*,
    assign::*,
    ident::*,
};

pub struct ParserStat {
    pub nodes: Nodes,
    pub errors: Errors,
    _style: Body,
}

impl ParserStat {
    pub fn init() -> Self {
        Self { nodes: Nodes::new(), errors: Vec::new() , _style: Body::Fun} 
    }
    pub fn style(&mut self, style: Body) {
        self._style = style;
    }
}
impl Parse for ParserStat {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { self.errors.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        while let Some(_) = lex.bump() {
            match self._style {
                Body::Top => {
                    if let Err(err) = self.parse_top(lex) { self.errors.push(err) }
                },
                Body::Typ => {
                },
                Body::Imp => {
                    if let Err(err) = self.parse_imp(lex) { self.errors.push(err) }
                },
                Body::Fun => {
                },
            }
        }
        Ok(())
    }
}

impl ParserStat {
    fn parse_top(&mut self, lex: &mut lexer::Elements) -> Vod {
        let token = lex.curr(true)?; lex.eat();
        if (lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
            && branch::body_top(lex, true)? 
        {
            let mut parser = ParserStatAss::init();
            if let Err(err) = parser.parse(lex) { self.errors.push(err) }
            self.nodes.extend(parser.nodes());
            self.errors.extend(parser.errors());

        }
        else if lex.curr(false)?.key().is_void() { return Ok(()); } 
        else if let Err(err) = check::unexpected_top(lex, token) { self.errors.push(err) }
        return Ok(());
    }

    fn parse_imp(&mut self, lex: &mut lexer::Elements) -> Vod {
        let token = lex.curr(true)?; lex.eat();
        if (lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
            && branch::body_imp(lex, true)? 
        {
            let mut parser = ParserStatAss::init();
            if let Err(err) = parser.parse(lex) { self.errors.push(err) }
            self.nodes.extend(parser.nodes());
            self.errors.extend(parser.errors());
        }
        else if lex.curr(false)?.key().is_void() { return Ok(()); } 
        else if let Err(err) = check::unexpected_imp(lex, token) { self.errors.push(err) }
        return Ok(());
    }
    fn parse_one(&mut self, lex: &mut lexer::Elements) -> Vod {
        match self._style {
            Body::Top => {
                let token = lex.curr(true)?; lex.eat();
                if (lex.curr(true)?.key().is_assign()
                    || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                    && branch::body_top(lex, true)? {
                    let mut parser = ParserStatAss::init();
                    if let Err(err) = parser.parse(lex) { return Err(err) }
                    // if let Err(err) = parser.parse(lex) { self.errors.push(err) }
                    self.nodes.extend(parser.nodes());
                    self.errors.extend(parser.errors());
                    return Ok(())
                } else if lex.curr(false)?.key().is_void() { return Ok(());
                } else { 
                    eater::until_term(lex, true)?;
                    return check::unexpected_top(lex, token); 
                }
            }
            Body::Typ => {
                let deep = lex.curr(false)?.loc().deep() - 1;
                loop{
                    let token = lex.curr(true)?; lex.eat();
                    if (matches!(lex.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) && lex.curr(false)?.loc().deep() == deep ) 
                        || lex.curr(false)?.key().is_eof() { 
                            return Ok(()) 
                    } 

                    if (lex.curr(true)?.key().is_assign() || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                        && branch::body_typ(lex, true)? {
                            let mut parser = ParserStatAss::init();
                            if let Err(err) = parser.parse(lex) { return Err(err) }
                            // if let Err(err) = parser.parse(lex) { self.errors.push(err) }
                            self.nodes.extend(parser.nodes());
                            self.errors.extend(parser.errors());
                            lex.eat(); lex.jump(0, false)?;
                    } else if lex.curr(false)?.key().is_void() { lex.jump(0, false)?;
                    } else { 
                        eater::until_term(lex, true)?;
                        return check::unexpected_typ(lex, token); 
                    }
                }
            }
            Body::Imp => {
                let deep = lex.curr(false)?.loc().deep() - 1;
                loop{
                    let token = lex.curr(true)?; lex.eat();
                    if (matches!(lex.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) && lex.curr(false)?.loc().deep() == deep ) 
                        || lex.curr(false)?.key().is_eof() { 
                            return Ok(()) 
                    } 

                    if (lex.curr(true)?.key().is_assign() || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                        && branch::body_imp(lex, true)? {
                            let mut parser = ParserStatAss::init();
                            if let Err(err) = parser.parse(lex) { return Err(err) }
                            // if let Err(err) = parser.parse(lex) { self.errors.push(err) }
                            self.nodes.extend(parser.nodes());
                            self.errors.extend(parser.errors());
                            lex.eat(); lex.jump(0, false)?;
                    } else if lex.curr(false)?.key().is_void() { lex.jump(0, false)?;
                    } else { 
                        eater::until_term(lex, true)?;
                        return check::unexpected_imp(lex, token); 
                    }
                }
            }
            Body::Fun => {
                let deep = lex.curr(false)?.loc().deep() - 1;
                loop{
                    if (matches!(lex.curr(false)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) && lex.curr(false)?.loc().deep() == deep ) 
                        || lex.curr(false)?.key().is_eof() { 
                            return Ok(()) 
                    } 

                    if (lex.curr(true)?.key().is_assign() || (matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_)) && lex.peek(0, true)?.key().is_assign()))
                        && branch::body_fun(lex, true)? {
                            let mut parser = ParserStatAss::init();
                            if let Err(err) = parser.parse(lex) { return Err(err) }
                            // if let Err(err) = parser.parse(lex) { self.errors.push(err) }
                            self.nodes.extend(parser.nodes());
                            self.errors.extend(parser.errors());
                            lex.eat(); lex.jump(0, false)?;
                    } else if lex.curr(false)?.key().is_void() { lex.jump(0, false)?;
                    } else { 
                        eater::expr_body2(lex)?;
                        return Ok(())
                    }
                }
            }
        }
    }
}

