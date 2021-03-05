use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssVar};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater};

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;

#[derive(Clone)]
pub struct ParserStatAssAli {
    pub nodes: Nodes,
    _alias: bool,
}

impl ParserStatAssAli {
    pub fn len(&self) -> usize { self.nodes.len() }
        pub fn init() -> Self {
        Self { nodes: Nodes::new(), _alias: true } 
    }
}
impl Parse for ParserStatAssAli {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssVar::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init();
        opts.parse(lex)?;

        // add "ali"
        node.set_string(lex.curr(true)?.con().to_string());
        lex.jump(0, false)?;

        // match options after var  -> "[opts]"
        opts.parse(lex)?;
        if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }
        check::expect_void(lex)?;

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init();
        idents.once();
        idents.parse(lex)?; lex.eat();
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init();
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect_many(lex, vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::void(VOID::endline_)
        ], true)?;
        lex.eat();

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        Ok(())
    }
}

impl ParserStatAssAli {
        pub fn extend(&mut self, n: Nodes) {
            self.nodes.extend(n)
        }
}
