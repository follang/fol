use crate::types::{Vod, Errors};
use crate::syntax::nodes::{Node, Nodes, NodeStatDecS};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, Body};
use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;
use crate::syntax::parse::expr::ParseExpr;

#[derive(Clone)]
pub struct ParserStatAssDef {
    pub nodes: Nodes,
    pub errors: Errors,
    _level: usize,
    _style: Body,
}

impl ParserStatAssDef {
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

impl Parse for ParserStatAssDef {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { Vec::new() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut loc = lex.curr(true)?.loc().clone();
        loc.set_deep(self.level() as isize);
        let mut node = NodeStatDecS::default();
        
        // match symbol before def  -> "~"
        let mut opts = ParserStatAssOpts::init();
        opts.parse(lex)?;

        // add "def"
        node.set_string(lex.curr(true)?.con().to_string());
        lex.jump(0, false)?;

        // match options after def  -> "[opts]"
        opts.parse(lex)?;
        if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }
        check::expect_void(lex)?;

        // match identifier/quoted string "ident" or "'quoted'"
        let mut idents = ParserStatIdent::init();
        idents.parse(lex)?;
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // Optional parameters for macro definitions like def 'name'(a: any)
        if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::RoundO)) {
            // TODO: Parse parameters for macro definitions
            // For now, skip parameters
            let mut paren_depth = 0;
            while let Ok(current) = lex.curr(true) {
                match current.key() {
                    KEYWORD::Symbol(SYMBOL::RoundO) => {
                        paren_depth += 1;
                        lex.jump(0, false)?;
                    }
                    KEYWORD::Symbol(SYMBOL::RoundC) => {
                        paren_depth -= 1;
                        lex.jump(0, false)?;
                        if paren_depth == 0 {
                            break;
                        }
                    }
                    _ => {
                        lex.jump(0, false)?;
                    }
                }
            }
        }

        // match datatypes after :  -> "int[opts][]" or "mac" or "alt" or "def[]"
        check::expect(lex, KEYWORD::Symbol(SYMBOL::Colon), true)?;
        let mut dt = ParserStatDatatypes::init(self.style());
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect(lex, KEYWORD::Symbol(SYMBOL::Equal), true)?;
        lex.jump(0, true)?;

        // match definition body (usually a quoted string for macros)
        let mut body = ParseExpr::init();
        body.parse(lex)?; 
        lex.eat();
        if body.nodes.len() > 0 { node.set_body(Some(body.nodes.get(0))); }

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        Ok(())
    }
}