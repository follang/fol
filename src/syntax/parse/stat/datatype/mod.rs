use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{eater, check};

pub use crate::syntax::nodes::stat::ident;

pub struct ParserStatDatatypes {
    pub nodes: Nodes,
    _inparam: bool,
}

impl ParserStatDatatypes {
    pub fn init(inparam: bool) -> Self {
        Self { 
            nodes: Nodes::new(),
            _inparam: inparam,
        } 
    }
}
impl Parse for ParserStatDatatypes {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // eat ":"
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            lex.jump(0, true)?; 
        } else {
            return Ok(())
        }

        while !lex.curr(true)?.key().is_eof() {
        // match type
            check::expect_ident(lex, true)?; lex.eat();
            let dt = ident::NodeStatIdent(lex.curr(true)?.con().to_string());
            let node = Node::new(Box::new(dt));
            self.nodes.push(node);
            lex.jump(0, false)?; 

            // match options after type  -> "[opts]"
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
                eater::until_bracket(lex)?;
            }

            // match restrictions after type  -> "[rest]"
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
                eater::until_bracket(lex)?;
            }
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::equal_)
                || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::semi_)
                || lex.curr(true)?.key().is_eol()
                || ((lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_)
                    || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_))
                    && self._inparam)
            {
                lex.eat();
                break
            } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::comma_) {
                lex.jump(0, true)?; lex.eat();
            }
            lex.eat();
        }

        Ok(())
    }
}
