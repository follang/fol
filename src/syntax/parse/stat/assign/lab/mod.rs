use crate::{syntax::token::{KEYWORD, SYMBOL}, types::{Vod, Errors}};
use crate::syntax::nodes::{Node, Nodes, NodeStatDecS};
use crate::syntax::lexer;
use super::Parse;

use crate::syntax::parse::{check, Body};
use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatAssLab {
    pub nodes: Nodes,
    pub errors: Errors,
    _style: Body,
    _level: usize,
}

impl ParserStatAssLab {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init(level: usize, style: Body) -> Self {
        Self {
            nodes: Nodes::new(),
            errors: Vec::new(),
            _level: level,
            _style: style.clone(),
        } 
    }
    pub fn level(&self) -> usize { self._level }
    pub fn style(&self) -> Body { self._style }
}
impl Parse for ParserStatAssLab {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { Vec::new() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut loc = lex.curr(true)?.loc().clone();
        loc.set_deep(self.level() as isize);
        let mut node = NodeStatDecS::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init();
        opts.parse(lex)?;

        // add "lab"
        node.set_string(lex.curr(true)?.con().to_string());
        lex.jump(0, false)?;

        // match options after var  -> "[opts]"
        opts.parse(lex)?;
        if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }
        check::expect_void(lex)?;

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init();
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        check::expect(lex, KEYWORD::Symbol(SYMBOL::Colon), true)?;
        let mut dt = ParserStatDatatypes::init(self.style());
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
        Ok(())
    }
}
