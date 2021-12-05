use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{eater, check, Body};

// pub mod when;
// pub mod loop;


pub struct ParserControlStat {
    pub nodes: Nodes,
    _colon: bool,
    _once: bool,
    _style: Body,
}

impl ParserControlStat {
    pub fn init(style: Body) -> Self {
        Self { 
            nodes: Nodes::new(),
            _colon: true,
            _once: false,
            _style: style,
        } 
    }
    pub fn nocolon(&mut self) { self._colon = false; }
    pub fn once(&mut self) { self._once = true; }
    pub fn style(&self) -> Body { self._style }
}
impl Parse for ParserControlStat {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { Vec::new() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
                Ok(())
    }
}
