use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssTyp};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater};

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::parameters::*;
use crate::syntax::parse::stat::generics::*;
use crate::syntax::parse::stat::datatype::*;

#[derive(Clone)]
pub struct ParserStatAssImp {
    pub nodes: Nodes,
}

impl ParserStatAssImp {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self { nodes: Nodes::new() } 
    }
}
impl Parse for ParserStatAssImp {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssTyp::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init(false);
        opts.parse(lex)?;

        // match "var"
        check::expect(lex,  KEYWORD::buildin(BUILDIN::imp_) , true)?;
        node.set_string(lex.curr(true)?.con().to_string());
        lex.jump(0, false)?;

        // match options after var  -> "[opts]"
        opts.parse(lex)?;
        if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }

        // match generics
        let mut gen = ParserStatGenerics::init();
        gen.parse(lex)?;
        if gen.nodes.len() > 0 { node.set_generics(Some(gen.nodes.clone())); }
        check::expect_void(lex)?;

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(true);
        idents.parse(lex)?;
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // match parameters after (  -> "(one, two)"
        let mut parameters = ParserStatParameters::init();
        parameters.parse(lex)?; lex.eat();
        if parameters.nodes.len() > 0 { node.set_parameters(Some(parameters.nodes.clone())) }

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(true);
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect(lex, KEYWORD::symbol(SYMBOL::equal_), true)?;
        lex.jump(0, true)?;

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        eater::expr_body(lex)?;
        // eater::until_term(lex, false)?;
        Ok(())
    }
}
