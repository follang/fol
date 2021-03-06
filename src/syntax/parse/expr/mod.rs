use crate::types::Vod;

use crate::syntax::nodes::Nodes;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::Body;



pub struct ParseExpr {
    pub nodes: Nodes,
    _style: Body,
}

impl ParseExpr {
    pub fn init() -> Self {
        Self { nodes: Nodes::new(), _style: Body::Top} 
    }
    pub fn style(&mut self, style: Body) {
        self._style = style;
    }
}
impl Parse for ParseExpr {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        lex.until_term(false)?;
        Ok(())
    }
}
