use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::assign::opts::*;

pub use crate::syntax::parse::stat::assign::opts::*;
pub use crate::syntax::parse::stat::ident::*;


pub struct ParserStatAssVar {
    pub nodes: Nodes,
}
impl std::default::Default for ParserStatAssVar {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for ParserStatAssVar {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut nodestatassvar = NodeStatAssVar::default();
        let mut opts = ParserStatAssOpts::default();
        if matches!(lex.curr(true).key(), KEYWORD::option(_) ) {
            if let KEYWORD::option(a) = lex.curr(true).key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(Box::new(assopt));
                opts.push(node);
            }
            lex.jump();
        }
        lex.expect( KEYWORD::assign(ASSIGN::var_) , true)?;
        lex.bump();
        if lex.curr(true).key() == KEYWORD::symbol(SYMBOL::squarO_) {
            opts.parse(lex)?;
            if opts.nodes.len() > 0 {
                nodestatassvar.set_options(Some(opts.nodes));
            }
        }
        lex.expect( KEYWORD::ident , true)?;

        lex.until_term();
        self.nodes.push(Node::new(Box::new(nodestatassvar)));
        Ok(())
    }
}
