use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::nodes::stat::assign::opts::*;

pub use crate::syntax::parse::stat::assign::opts::*;
pub use crate::syntax::parse::stat::ident::*;


pub struct ParserStatAssVar {
    pub nodes: Nodes,
    _source: Source,
    _recurse: bool
}

impl ParserStatAssVar {
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src, _recurse: false } 
    }
    pub fn len(&self) -> usize { self.nodes.len() }
}
impl Parse for ParserStatAssVar {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut nodestatassvar = NodeStatAssVar::default();
        let mut opts = ParserStatAssOpts::init(self._source.clone());
        let loc = lex.curr(true)?.loc().clone();
        // match symbol before var  -> "~"
        if matches!(lex.curr(true)?.key(), KEYWORD::option(_) ) {
            if let KEYWORD::option(a) = lex.curr(true)?.key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(lex.curr(true)?.loc().clone(), Box::new(assopt));
                opts.push(node);
            }
            lex.jump(0, true)?;
        }

        // match "var"
        lex.expect( KEYWORD::assign(ASSIGN::var_) , true)?;
        lex.jump(0, false)?;

        // match options after var  -> "[opts]"
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
            opts.parse(lex)?;
        }

        // match space after "var" or after "[opts]"
        lex.expect_void()?;
        lex.jump(0, false)?;

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(self._source.clone());
        idents.parse(lex)?;

        for i in 0..idents.nodes.len() {
            if opts.nodes.len() > 0 {
                nodestatassvar.set_options(Some(opts.nodes.clone()));
            }
            let mut newnode = nodestatassvar.clone();
            newnode.set_ident(Some(idents.nodes.get(i).clone()));
            self.nodes.push(Node::new(loc.clone(), Box::new(newnode)));
        }
        // if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::semi_)
        //     || lex.curr(true)?.key().is_eol() { return Ok(()) }
        lex.until_term(false)?;
        Ok(())
    }
}
