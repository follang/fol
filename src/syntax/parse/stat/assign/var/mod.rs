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
    _recurse: bool
}
impl std::default::Default for ParserStatAssVar {
    fn default() -> Self { Self { nodes: Nodes::new(), _recurse: false } }
}

impl Parse for ParserStatAssVar {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut nodestatassvar = NodeStatAssVar::default();
        let mut opts = ParserStatAssOpts::default();
        if matches!(lex.curr(true)?.key(), KEYWORD::option(_) ) {
            if let KEYWORD::option(a) = lex.curr(true)?.key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(lex.curr(true)?.loc().clone(), Box::new(assopt));
                opts.push(node);
            }
            lex.jump(0, true)?;
        }
        let loc = lex.curr(true)?.loc().clone();
        lex.expect( KEYWORD::assign(ASSIGN::var_) , true)?;
        lex.jump(0, false)?;
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
            opts.parse(lex)?;
            if opts.nodes.len() > 0 {
                nodestatassvar.set_options(Some(opts.nodes));
            }
        }
        let mut idents = ParserStatIdent::default();
        idents.parse(lex)?;

        lex.until_term(true)?;
        self.nodes.push(Node::new(loc, Box::new(nodestatassvar)));
        Ok(())
    }
}
