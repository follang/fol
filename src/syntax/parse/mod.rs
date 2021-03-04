use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes};
use crate::syntax::token::*;
use crate::syntax::lexer;

pub mod check;
pub mod eater;

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
        while let Some(e) = lex.bump() {
            if let Err(err) = parser.parse(lex) {
                parser.errors.push(err)
            }
        }
        println!();
        nodinter!(parser.nodes.clone());
        errinter!(parser.errors.clone());
        parser
    }
}

impl Parse for Parser {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key().is_comment() {
            lex.jump(0, true)?;
        }
        if lex.curr(true)?.key().is_assign()
            || (matches!(lex.curr(true)?.key(), KEYWORD::symbol(_)) && lex.peek(0, true)?.key().is_assign())
        {
            let mut parse_stat = ParserStat::init();
            match parse_stat.parse(lex) {
                Ok(()) => { self.nodes.extend(parse_stat.nodes) },
                Err(err) => { self.errors.push(err) }
            }
        } else {
            // check::expect(lex,  KEYWORD::assign(ASSIGN::ANY) , true)?;
            lex.until_term(false)?;
        }
        Ok(())
    }
}
