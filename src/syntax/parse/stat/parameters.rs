use crate::types::{Vod, Con, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssVar};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatParameters {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStatParameters {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStatParameters {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        lex.jump(0, true)?;
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_) {
            lex.jump(0, true)?;
            return Ok(())
        }

        while !lex.curr(true)?.key().is_eof() {
            match self.parse2(lex) {
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
        Ok(())
    }
}
impl ParserStatParameters {
    fn parse2(&mut self, lex: &mut lexer::Elements) -> Con<Nodes> {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssVar::default();
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

        // match "var"
        if lex.curr(true)?.key() == KEYWORD::assign(ASSIGN::var_) {
            lex.jump(0, true)?;
        }

        // match options after var  -> "[opts]"
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
            opts.parse(lex)?;
            if opts.nodes.len() > 0 {
                node.set_options(Some(opts.nodes.clone()));
            }
            // match space after "var" or after "[opts]"
            lex.expect_void()?;
            lex.jump(0, false)?;

        }

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(self._source.clone());
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(self._source.clone(), true);
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            dt.parse(lex)?;
        }
        lex.expect_many(vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::symbol(SYMBOL::equal_),
            KEYWORD::symbol(SYMBOL::roundC_),
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

        let mut nodes: Nodes = List::new();

        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                node.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            node.set_ident(Some(idents.nodes.get(i).clone()));
            let mut newnode = Node::new(Box::new(node.clone()));
            newnode.set_loc(loc.clone());
            nodes.push(newnode);
        }
        lex.until_key(vec![KEYWORD::symbol(SYMBOL::roundC_), KEYWORD::symbol(SYMBOL::semi_)])?;
        Ok(nodes)
    }
}

impl ParserStatParameters {
    pub fn extend(&mut self, n: Nodes) {
        self.nodes.extend(n)
    }
}
