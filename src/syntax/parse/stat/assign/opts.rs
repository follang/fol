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
        lex.jump(0, false)?;
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_) {
            lex.jump(0, true)?;
            return Ok(())
        }
        loop {
            lex.expect_option(true)?;
            if let KEYWORD::option(a) = lex.curr(true)?.key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(lex.curr(true)?.loc().clone(), Box::new(assopt));
                self.nodes.push(node);
            }
            lex.jump(0, true)?;
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_)
                || lex.curr(true)?.key().is_eol()
            {
                lex.jump(0, true)?;
                return Ok(())
            } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::comma_) {
                if lex.peek(0, true)?.key() == KEYWORD::symbol(SYMBOL::squarC_) 
                {
                    lex.jump(1, true)?;
                    return Ok(())
                }
                lex.jump(0, false)?;
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
