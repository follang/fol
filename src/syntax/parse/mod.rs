use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;

pub mod stat;
pub use crate::syntax::parse::stat::*;
pub mod expr;

pub trait Parse {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod;
}

pub struct Parser {
    pub nodes: Nodes,
    pub errors: Errors,
    _source: Source,
}

impl Parser {
    pub fn init (lex: &mut lexer::Elements, src: Source) -> Self {
        let mut parser = Self { nodes: Nodes::new(), errors: Vec::new(), _source: Source::default() };
        parser._source = src.clone();
        while let Some(e) = lex.bump() {
            // lex.debug().ok();
            if let Err(err) = parser.parse(lex) {
                parser.errors.push(err)
            }
        }
        logit!(parser.nodes.len().to_string());
        for e in parser.nodes.clone() {
            println!("{}, {}", e.loc().unwrap().print(&parser._source), e);
        };
        printer!(parser.errors.clone());
        parser
    }
}

impl Parse for Parser {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if matches!(lex.curr(false)?.key(), KEYWORD::assign(_))
            || (matches!(lex.curr(false)?.key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false)?.key(), KEYWORD::assign(_)))
        {
            let mut parse_stat = ParserStat::init(self._source.clone());
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
