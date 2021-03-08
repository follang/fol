use crate::types::{Vod, Con, List, Errors};
use crate::syntax::nodes::{Node, Nodes, NodeStatDecS};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

use crate::syntax::parse::{eater, check};
use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatParameters {
    pub nodes: Nodes,
    errors: Errors,
}

impl ParserStatParameters {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self {
            nodes: Nodes::new(),
            errors: Vec::new(),
        } 
    }
}
impl Parse for ParserStatParameters {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { self.errors.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::RoundO) {
            lex.jump(0, true)?;
        } else {
            return Ok(())
        }

        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::RoundC) {
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
                } else if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::RoundC) {
                    lex.jump(0, true)?;
                    break
                }
            }
        }
        Ok(())
    }
}
impl ParserStatParameters {
    fn parse_each(&mut self, lex: &mut lexer::Elements) -> Con<Nodes> {
        let mut loc = lex.curr(true)?.loc().clone();
        loc.set_deep(0);
        let mut node = NodeStatDecS::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init();
        opts.parse(lex)?;

        // match "var"
        if lex.curr(true)?.con() == "var" {
            lex.jump(0, true)?;
        }

        // match options after var  -> "[opts]"
        opts.parse(lex)?;
        if opts.nodes.len() > 0 { 
            node.set_options(Some(opts.nodes.clone()));
            check::expect_void(lex)?;
        }

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init();
        idents.once();
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init();
        dt.parse(lex)?;

        check::expect_many(lex, vec![ 
            KEYWORD::Symbol(SYMBOL::Semi),
            KEYWORD::Symbol(SYMBOL::Equal),
            KEYWORD::Symbol(SYMBOL::RoundC),
            KEYWORD::Void(VOID::EndLine)
        ], true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        let mut nodes: Nodes = List::new();
        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                node.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            node.set_ident(Some(idents.nodes.get(i).clone()));
            let id = Node::new(Box::new(node.clone()));
            // id.set_loc(loc.clone());
            nodes.push(id);
        }
        eater::until_key(lex, vec![KEYWORD::Symbol(SYMBOL::RoundC), KEYWORD::Symbol(SYMBOL::Semi)])?;
        Ok(nodes)
    }
}
