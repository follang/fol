use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssFun};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::contracts::*;
use crate::syntax::parse::stat::datatype::*;

#[derive(Clone)]
pub struct ParserStatAssFun {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStatAssFun {
    pub fn len(&self) -> usize { self.nodes.len() }
        pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStatAssFun {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssFun::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init(self._source.clone());
        if matches!(lex.curr(true)?.key(), KEYWORD::option(_) ) {
            if let KEYWORD::option(a) = lex.curr(true)?.key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(Box::new(assopt));
                opts.push(node);
            }
            lex.jump(0, true)?;
        }

        // match "typ"
        lex.expect( KEYWORD::assign(ASSIGN::fun_) , true)?;
        lex.jump(0, false)?;

        // match options after var  -> "[opts]"
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
            opts.parse(lex)?;
            if opts.nodes.len() > 0 {
                node.set_options(Some(opts.nodes.clone()));
            }
        }

        // match space after "var" or after "[opts]"
        lex.expect_void()?;
        lex.jump(0, false)?;

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(self._source.clone());
        idents.only_one();
        idents.parse(lex)?; lex.eat();

        // match parameters after (  -> "(one, two)"
        let mut parameters = ParserStatContract::init(self._source.clone());
        parameters.parse(lex)?; lex.eat();
        if parameters.nodes.len() > 0 { node.set_parameters(Some(parameters.nodes.clone())) }

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(self._source.clone());
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            dt.parse(lex)?;
        }

        lex.expect_many(vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::symbol(SYMBOL::equal_),
            KEYWORD::void(VOID::endline_)
        ], true)?;

        if dt.nodes.len() > idents.nodes.len() {
            return Err( catch!( Typo::ParserTypeDisbalance {
                msg: Some(format!(
                    "number of identifiers: [{}] is smaller than number of types [{}]",
                    idents.nodes.len(),
                    dt.nodes.len(),
                    )),
                loc: Some(loc.clone()), 
                src: self._source.clone(),
            }))
        }

        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                node.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            node.set_ident(Some(idents.nodes.get(i).clone()));
            let mut newnode = Node::new(Box::new(node.clone()));
            newnode.set_loc(loc.clone());
            self.nodes.push(newnode);
        }
        lex.until_term(false)?;
        Ok(())
    }
}

impl ParserStatAssFun {
        pub fn extend(&mut self, n: Nodes) {
            self.nodes.extend(n)
        }
}
