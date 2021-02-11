use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::datatype::*;

pub struct ParserStatDatatypes {
    pub nodes: Nodes,
    _source: Source,
}

impl ParserStatDatatypes {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src } 
    }
}
impl Parse for ParserStatDatatypes {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // eat "["
        lex.jump(0, false)?; 

        // match type
        lex.expect_types(true)?; lex.eat();
        if let KEYWORD::types(a) = lex.curr(true)?.key() {
            let dt: datatype::NodeExprDatatype = a.into();
            let node = Node::new(Box::new(dt));
            self.nodes.push(node);
        }
        lex.jump(0, false)?; 
        // lex.debug().ok();


        Ok(())
    }
}
