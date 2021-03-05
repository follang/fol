use crate::types::{Vod, Con, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssVar};
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
}

impl ParserStatParameters {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self { nodes: Nodes::new()} 
    }
}
impl Parse for ParserStatParameters {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundO_) {
            lex.jump(0, true)?;
        } else {
            return Ok(())
        }

        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_) {
            lex.jump(0, true)?;
            return Ok(())
        } else {
            while !lex.curr(true)?.key().is_eof() {
                match self.parse_each(lex) {
                    Ok(ok) => self.nodes.extend(ok),
                    Err(err) => return Err(err)
                };
                if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::semi_) {
                    lex.jump(0, true)?;
                } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_) {
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
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssVar::default();
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
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init();
        dt.parse(lex)?;

        check::expect_many(lex, vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::symbol(SYMBOL::equal_),
            KEYWORD::symbol(SYMBOL::roundC_),
            KEYWORD::void(VOID::endline_)
        ], true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        let mut nodes: Nodes = List::new();
        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                node.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            node.set_ident(Some(idents.nodes.get(i).clone()));
            let mut id = Node::new(Box::new(node.clone()));
            id.set_loc(loc.clone());
            nodes.push(id);
        }
        eater::until_key(lex, vec![KEYWORD::symbol(SYMBOL::roundC_), KEYWORD::symbol(SYMBOL::semi_)])?;
        Ok(nodes)
    }
}
