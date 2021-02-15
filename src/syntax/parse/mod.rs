use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes};
use crate::syntax::token::*;
use crate::syntax::lexer;

pub mod check;

pub mod stat;
pub mod expr;
pub use crate::syntax::parse::stat::*;

pub trait Parse {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod;
    fn nodes(&self) -> Nodes;
}

pub struct Parser {
    pub nodes: Nodes,
    pub errors: Errors,
}

impl Parser {
    pub fn init (lex: &mut lexer::Elements) -> Self {
        let mut parser = Self { nodes: Nodes::new(), errors: Vec::new() };
        let src = lex.peek(0, false).unwrap().loc().source().unwrap();
        while let Some(e) = lex.bump() {
            if let Err(err) = parser.parse(lex) {
                parser.errors.push(err)
            }
        }
        logit!(src.path(false));
        nodinter!(parser.nodes.clone());
        errinter!(parser.errors.clone());
        parser
    }
}

impl Parse for Parser {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false)?.key(), KEYWORD::assign(_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(_)))
        {
            let mut parse_stat = ParserStat::init();
            match parse_stat.parse(lex) {
                Ok(()) => { self.nodes.extend(parse_stat.nodes) },
                Err(err) => { self.errors.push(err) }
            }
        } else {
            lex.until_term(false)?;
        }
        Ok(())
    }
}
