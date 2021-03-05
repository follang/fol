use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssVar};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::check;

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatAssLab {
    pub nodes: Nodes,
}

impl ParserStatAssLab {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self { nodes: Nodes::new() } 
    }
}
impl Parse for ParserStatAssLab {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssVar::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init(false);
        opts.parse(lex)?;

        // add "lab"
        node.set_string(lex.curr(true)?.con().to_string());
        lex.jump(0, false)?;

        // match options after var  -> "[opts]"
        opts.parse(lex)?;
        if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }
        check::expect_void(lex)?;

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(true);
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(true);
        dt.parse(lex)?;

        check::expect_terminal(lex)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                node.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            node.set_ident(Some(idents.nodes.get(i).clone()));
            let mut id = Node::new(Box::new(node.clone()));
            id.set_loc(loc.clone());
            self.nodes.push(id);
        }
        lex.until_term(false)?;
        Ok(())
    }
}
