use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;


pub struct ParserStatAssVar {
    pub nodes: Nodes,
}
impl std::default::Default for ParserStatAssVar {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for ParserStatAssVar {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let varstat: VarStat = VarStat::default();
        if matches!(lex.curr(true).key(), KEYWORD::option(_) ) {
            lex.bump();
        }
        lex.expect_assign(true)?;
        lex.bump();
        if lex.curr(true).key() == KEYWORD::symbol(SYMBOL::squarO_) {
            lex.bump();
        }
        lex.expect_one( KEYWORD::option(OPTION::mut_) , true)?;



        lex.toend();
        self.nodes.push(Node::new(Box::new(varstat)));
        Ok(())
    }
}
