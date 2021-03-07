use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::check;

pub use crate::syntax::nodes::stat::ident;

pub struct ParserStatAssOpts {
    pub nodes: Nodes,
}

impl ParserStatAssOpts {
    pub fn init() -> Self {
        Self { 
            nodes: Nodes::new(),
        } 
    }
}
impl Parse for ParserStatAssOpts {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { Vec::new() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // eat "["
        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarO) {
            lex.jump(0, false)?;
        // match symbol before var  -> "~" and return 
        } else if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(_) )
            && !matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::Equal) )
        {
            let assopt = NodeStatIdent::new(lex.curr(true)?.con().clone());
            let node = Node::new(Box::new(assopt));
            self.nodes.push(node);
            lex.jump(0, true)?;
            return Ok(())
        } else {
            return Ok(())
        }

        // match "]" if there and return
        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarC) {
            lex.jump(0, true)?;
            return Ok(())
        }
        while !lex.curr(true)?.key().is_eof() {
            check::expect_ident_literal(lex, true)?;

            let reviver = NodeStatIdent::new(lex.curr(true)?.con().clone());
            let node = Node::new(Box::new(reviver));
            self.nodes.push(node);
            lex.jump(0, true)?;

            if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarC)
                || lex.curr(true)?.key().is_eol()
            {
                lex.jump(0, true)?;
                break
            } else if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::Comma) {
                if lex.peek(0, true)?.key() == KEYWORD::Symbol(SYMBOL::SquarC) 
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
