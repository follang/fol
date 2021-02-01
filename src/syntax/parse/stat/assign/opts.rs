use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::assign::opts::*;

pub struct ParserStatAssOpts {
    pub nodes: Nodes,
}
impl std::default::Default for ParserStatAssOpts {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for ParserStatAssOpts {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        lex.bump();
        if lex.curr(true).key() == KEYWORD::symbol(SYMBOL::squarC_) {
            return Ok(())
        }
        loop {
            lex.expect_option(true)?;
            if let KEYWORD::option(a) = lex.curr(true).key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(Box::new(assopt));
                self.nodes.push(node);
            }
            lex.jump();
            if lex.curr(true).key() == KEYWORD::symbol(SYMBOL::squarC_)
                || lex.curr(true).key().is_eol()
            {
                lex.bump();
                return Ok(())
            } else if lex.curr(true).key() == KEYWORD::symbol(SYMBOL::comma_) {
                lex.bump();
            }
        }
        // lex.until_bracket();
        // Ok(())
    }
}

impl ParserStatAssOpts {
    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }
}
