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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Body{
    Top,
    Fun,
    Typ,
    Imp,
    Par,
}

pub struct Parser {
    nodes: Nodes,
    errors: Errors,
}

impl Parser {
    pub fn init (lex: &mut lexer::Elements) -> Self {
        let mut parser = Self {
            nodes: Nodes::new(),
            errors: Vec::new(),
        };
        let mut stat = ParserStat::init(Body::Top, 1);
        if let Err(err) = stat.parse(lex) { 
            parser.errors.push(err)
        }
        parser.nodes.extend(stat.nodes());
        parser.errors.extend(stat.errors());
        parser
    }
    pub fn nodes(&self) -> Nodes { self.nodes.clone() }
    pub fn errors(&self) -> Errors { self.errors.clone() }
}
