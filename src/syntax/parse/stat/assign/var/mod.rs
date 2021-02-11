use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::parse::stat::assign::opts::*;
pub use crate::syntax::parse::stat::ident::*;
pub use crate::syntax::parse::stat::datatype::*;


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
        let loc = lex.curr(true)?.loc().clone();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init(self._source.clone());
        if matches!(lex.curr(true)?.key(), KEYWORD::option(_) ) {
            if let KEYWORD::option(a) = lex.curr(true)?.key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(Box::new(assopt));
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
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(self._source.clone());
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            dt.parse(lex)?;
        }
        lex.debug().ok();

        for i in 0..idents.nodes.len() {
            let mut nodestatassvar = NodeStatAssVar::default();
            if opts.nodes.len() > 0 {
                nodestatassvar.set_options(Some(opts.nodes.clone()));
            }
            if dt.nodes.len() > 0 {
                nodestatassvar.set_datatype(Some(dt.nodes.get(0).clone()));
            }
            let mut nodestatassvar_new = nodestatassvar.clone();
            nodestatassvar_new.set_ident(Some(idents.nodes.get(i).clone()));
            let mut newnode = Node::new(Box::new(nodestatassvar_new));
            newnode.set_loc(loc.clone());
            self.nodes.push(newnode);
        }
        // if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::semi_)
        //     || lex.curr(true)?.key().is_eol() { return Ok(()) }
        lex.until_term(false)?;
        Ok(())
    }
}
