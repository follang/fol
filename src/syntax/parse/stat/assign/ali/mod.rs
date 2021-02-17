use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssAli};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater};

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::contracts::*;
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
        let mut node = NodeStatAssAli::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init(false);
        opts.parse(lex)?;

        // match "typ"
        check::expect(lex, KEYWORD::assign(ASSIGN::ali_) , true)?;
        lex.jump(0, false)?;

        // match options after var  -> "[opts]"
        opts.parse(lex)?;
        if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(true);
        if lex.curr(true)?.key().is_type() {
            idents.parse_2(lex)?; lex.eat();
            self._alias = false;
        } else {
            idents.only_one();
            idents.parse(lex)?; lex.eat();
        }
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // match contracts after (  -> "(one, two)"
        let mut contracts = ParserStatContract::init();
        contracts.parse(lex)?; lex.eat();
        if contracts.nodes.len() > 0 { node.set_contracts(Some(contracts.nodes.clone())) }

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(true);
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect_many(lex, vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::symbol(SYMBOL::equal_),
            KEYWORD::void(VOID::endline_)
        ], true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        eater::until_term(lex, false)?;
        Ok(())
    }
}

impl ParserStatAssAli {
        pub fn extend(&mut self, n: Nodes) {
            self.nodes.extend(n)
        }
}
