use crate::types::*;
use crate::syntax::nodes::Nodes;
// use crate::syntax::token::*;
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

pub enum Body{
    Top,
    Fun,
    Typ,
    Imp,
}

pub struct Parser {
    pub nodes: Nodes,
    pub errors: Errors,
}

impl Parser {
    pub fn init (lex: &mut lexer::Elements) -> Self {
        let mut parser = Self { nodes: Nodes::new(), errors: Vec::new() };
        while let Some(_) = lex.bump() {
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
        } else {
            let mut parser = ParserStat::init();
            parser.style(Body::Top);
            match parser.parse(lex) {
                Ok(()) => { self.nodes.extend(parser.nodes) },
                Err(err) => { self.errors.push(err) }
            }
        }
        Ok(())
    }
}
