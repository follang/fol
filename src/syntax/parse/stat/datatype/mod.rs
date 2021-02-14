use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::check;

pub use crate::syntax::nodes::stat::datatype::*;

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
        lex.jump(0, false)?; 

        while !lex.curr(true)?.key().is_eof() {
        // match type
            check::expect_types(lex, true)?; lex.eat();
            if let KEYWORD::types(a) = lex.curr(true)?.key() {
                let dt: datatype::NodeExprDatatype = a.into();
                let node = Node::new(Box::new(dt));
                self.nodes.push(node);
            }
            lex.jump(0, false)?; 

            // match options after type  -> "[opts]"
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
                check::until_bracket(lex)?;
            }

            // match restrictions after type  -> "[rest]"
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
                check::until_bracket(lex)?;
            }
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::equal_)
                || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::semi_)
                || lex.curr(true)?.key().is_eol()
                || (lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_) && self._inparam)
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
