use crate::types::*;
use crate::syntax::nodes::Nodes;
// use crate::syntax::token::*;
use crate::syntax::lexer;

pub mod check;
pub mod eater;
pub mod branch;

pub mod stat;
pub mod expr;
pub use crate::syntax::parse::stat::*;

pub trait Parse {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod;
    fn nodes(&self) -> Nodes;
    fn errors(&self) -> Errors;
}

pub enum Body{
    Top,
    Fun,
    Typ,
    Imp,
}

pub struct Parser {
    nodes: Nodes,
    errors: Errors,
}

impl Parser {
    pub fn init (lex: &mut lexer::Elements) -> Self {
        let mut parser = Self { nodes: Nodes::new(), errors: Vec::new() };
        if let Err(err) = parser.parse(lex) { parser.errors.push(err) }
        println!();
        noditer!(parser.nodes.clone());
        erriter!(parser.errors.clone());
        parser
    }
}

impl Parse for Parser {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { self.errors.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // while let Some(_) = lex.bump() {
            let mut parser = ParserStat::init(Body::Top, 1);
            parser.parse(lex).ok();
            self.nodes.extend(parser.nodes());
            self.errors.extend(parser.errors());
        // }
        Ok(())
    }
}
