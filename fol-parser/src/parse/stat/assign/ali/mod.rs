use fol_types::{Vod, Errors};
use crate::nodes::{Node, Nodes, NodeStatDecS};
use crate::token::*;
use fol_lexer;
use super::Parse;

use crate::parse::{check, Body};
use crate::parse::stat::assign::opts::*;
use crate::parse::stat::ident::*;
use crate::parse::stat::datatype::*;

#[derive(Clone)]
pub struct ParserStatAssAli {
    pub nodes: Nodes,
    pub errors: Errors,
    _alias: bool,
    _level: usize,
    _style: Body,
}

impl ParserStatAssAli {
    pub fn len(&self) -> usize { self.nodes.len() }
        pub fn init(level: usize, style: Body) -> Self {
        Self {
            nodes: Nodes::new(),
            errors: Vec::new(),
            _alias: true,
            _level: level,
            _style: style.clone(),
        } 
    }
    pub fn level(&self) -> usize { self._level }
    pub fn style(&self) -> Body { self._style }
}
impl Parse for ParserStatAssAli {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { Vec::new() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut loc = lex.curr(true)?.loc().clone();
        loc.set_deep(self.level() as isize);
        let mut node = NodeStatDecS::default();
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
        let mut dt = ParserStatDatatypes::init(self.style());
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect_many(lex, vec![ 
            KEYWORD::Symbol(SYMBOL::Semi),
            KEYWORD::Void(VOID::EndLine)
        ], true)?;
        lex.eat();

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        Ok(())
    }
}
