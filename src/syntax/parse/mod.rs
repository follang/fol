use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;

pub mod stat;
pub use crate::syntax::parse::stat::*;
pub mod expr;

pub trait Parse {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod;
}

pub trait Fixer {
    fn fix(&mut self, lex: &mut lexer::Elements) -> Vod;
}

pub struct Parser {
    pub nodes: Nodes,
    pub errors: Errors
}
impl std::default::Default for Parser {
    fn default() -> Self { Self { nodes: Nodes::new(), errors: Vec::new() } }
}

impl Parser {
    pub fn init (&mut self, lex: &mut lexer::Elements) {
        lex.bump();
        while let Some(e) = lex.next() {
            // lex.debug();
            if let Err(err) = self.parse(lex) {
                self.errors.push(err)
            }
        }
        printer!(self.errors.clone());
        for e in self.nodes.clone() {
            println!("{}, {}", e.loc().unwrap().show(), e);
        }
    }
}

impl Parse for Parser {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false)?.key(), KEYWORD::assign(_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(_)))
        {
            let mut parse_stat = ParserStat::default();
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
