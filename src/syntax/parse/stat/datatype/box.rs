use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{eater, check};
use crate::syntax::nodes::stat::datatype;
use crate::syntax::parse::stat::datatype::ParserStatDatatypes;



pub struct ParserStatData {
    pub nodes: Nodes,
}

impl ParserStatData {
    pub fn init() -> Self {
        Self { 
            nodes: Nodes::new(),
        } 
    }
}


impl Parse for ParserStatData {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut node = datatype::NodeStatDatatypes::default();
        // match type
        check::expect_ident(lex, true)?; lex.eat();
        node.set_string(lex.curr(true)?.con().to_string());
        lex.jump(0, false)?; 

        // match options after type  -> "[opts]"
        let mut op = ParserStatDatatypes::init(false);
        op.parse(lex)?;
        if op.nodes.len() > 0 { node.set_form(Some(op.nodes.clone())); }

        //eat "]"
        check::expect(lex, KEYWORD::symbol(SYMBOL::squarC_), true)?;
        lex.jump(0, false)?; 

        // match restrictions after type  -> "[rest]"
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
            eater::until_bracket(lex)?;
        }
        let id = Node::new(Box::new(node));
        self.nodes.push(id);
        Ok(())
    }
}


