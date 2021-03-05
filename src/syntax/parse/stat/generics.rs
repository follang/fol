use crate::types::{Vod, Con, List};
use crate::syntax::nodes::{Node, Nodes, NodeStatDecS};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

use crate::syntax::parse::{eater, check};
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatGenerics {
    pub nodes: Nodes,
}

impl ParserStatGenerics {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self { nodes: Nodes::new()} 
    }
}
impl Parse for ParserStatGenerics {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarO) {
            lex.jump(0, true)?;
        } else {
            return Ok(())
        }

        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarC) {
            lex.jump(0, true)?;
            return Ok(())
        } else {
            while !lex.curr(true)?.key().is_eof() {
                match self.parse_each(lex) {
                    Ok(ok) => self.nodes.extend(ok),
                    Err(err) => return Err(err)
                };
                if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::Semi) {
                    lex.jump(0, true)?;
                } else if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarC) {
                    lex.jump(0, true)?;
                    break
                }
            }
        }
        Ok(())
    }
}
impl ParserStatGenerics {
    fn parse_each(&mut self, lex: &mut lexer::Elements) -> Con<Nodes> {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatDecS::default();

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init();
        idents.parse(lex)?; lex.eat();
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init();
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect_many(lex, vec![ 
            KEYWORD::Symbol(SYMBOL::Semi),
            KEYWORD::Symbol(SYMBOL::SquarC),
        ], true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        let nodes: Nodes = List::new();
        let mut id = Node::new(Box::new(node));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        eater::until_key(lex, vec![KEYWORD::Symbol(SYMBOL::SquarC), KEYWORD::Symbol(SYMBOL::Semi)])?;
        Ok(nodes)
    }
}
