use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub use crate::syntax::parse::stat::assign::opts::*;
pub use crate::syntax::parse::stat::ident::*;
pub use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
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
    pub fn recurse(&mut self) { self._recurse = true }
}
impl Parse for ParserStatAssVar {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut nodestatassvar = NodeStatAssVar::default();
        // match symbol before var  -> "~"
        if !self._recurse {
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
            if opts.nodes.len() > 0 {
                nodestatassvar.set_options(Some(opts.nodes.clone()));
            }

            // match space after "var" or after "[opts]"
            lex.expect_void()?;
            lex.jump(0, false)?;

            // march "(" to go recursively
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundO_) {
                lex.jump(0, true)?; lex.eat();
                let mut nodes: Nodes = List::new();
                while !lex.curr(true)?.key().is_eof() {
                    let mut newself = self.clone();
                    newself.recurse();
                    lex.expect( KEYWORD::ident , true)?; lex.eat();
                    newself.parse(lex)?;
                    nodes.extend(newself.nodes);

                    if lex.curr(true)?.key().is_terminal() { lex.jump(0, true)? };
                    if matches!(lex.curr(true)?.key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                        lex.jump(0, true)?;
                        lex.expect( KEYWORD::void(VOID::endline_) , true)?; 
                        lex.jump(0, true)?;
                        break
                    }
                }
                self.nodes.extend(nodes);
                return Ok(());
            }
        }

        lex.debug().ok();

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(self._source.clone());
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(self._source.clone());
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            dt.parse(lex)?;
        }

        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                nodestatassvar.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            nodestatassvar.set_ident(Some(idents.nodes.get(i).clone()));
            let mut newnode = Node::new(Box::new(nodestatassvar.clone()));
            newnode.set_loc(loc.clone());
            self.nodes.push(newnode);
        }
        lex.until_term(false)?;
        // printer!(self.nodes.clone());
        Ok(())
    }
}

impl ParserStatAssVar {
        pub fn extend(&mut self, n: Nodes) {
            self.nodes.extend(n)
        }
}
