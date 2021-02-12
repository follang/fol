use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::assign::opts::*;

pub struct ParserStatAssOpts {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStatAssOpts {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStatAssOpts {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // eat "["
        lex.jump(0, false)?;

        // match "]" if there and return
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_) {
            lex.jump(0, true)?;
            return Ok(())
        }
        while !lex.curr(true)?.key().is_eof() {
            lex.expect_option(true)?;
            if let KEYWORD::option(a) = lex.curr(true)?.key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(Box::new(assopt));
                self.nodes.push(node);
            }
            lex.jump(0, true)?;

            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_)
                || lex.curr(true)?.key().is_eol()
            {
                lex.jump(0, true)?;
                break
            } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::comma_) {
                if lex.peek(0, true)?.key() == KEYWORD::symbol(SYMBOL::squarC_) 
                {
                    lex.jump(1, true)?;
                    break
                }
                lex.jump(0, false)?;
            }
        }
        Ok(())
    }
}

impl ParserStatAssOpts {
    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }
}
