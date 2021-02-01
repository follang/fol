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

pub struct Parser {
    pub nodes: Nodes,
    pub errors: Errors
}
impl std::default::Default for Parser {
    fn default() -> Self { Self { nodes: Nodes::new(), errors: Vec::new() } }
}

impl Parse for Parser {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if let Some(val) = lex.bump() { if let Err(e) = val { crash!(e) }; };
        while let Some(val) = lex.bump() {
            if matches!(lex.curr(false).key(), KEYWORD::assign(_))
                || (matches!(lex.curr(false).key(), KEYWORD::option(_))
                    && matches!(lex.peek(0, false).key(), KEYWORD::assign(_)))
            {
                let mut parse_stat = ParserStat::default();
                match parse_stat.parse(lex) {
                    Ok(()) => { self.nodes.extend(parse_stat.nodes) },
                    Err(err) => { self.errors.push(err) }
                }
            } else {
                lex.until_term(false);
            }
        }
        printer!(self.errors.clone());
        println!("\n\n--------------------------------------------------\n\n");
        for e in self.nodes.clone() {
            println!("{}, {}", e.loc().unwrap(), e);
        }
        Ok(())
    }
}
