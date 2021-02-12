use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssVar};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatAssVar {
    pub nodes: Nodes,
    _source: Source,
    _recurse: bool,
    _oldstat: NodeStatAssVar,
}

impl ParserStatAssVar {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init(src: Source) -> Self {
        Self { nodes: Nodes::new(), _source: src, _recurse: false, _oldstat: NodeStatAssVar::default() } 
    }
    pub fn recurse(&self) -> Self {
        let mut new_clone = self.clone();
        new_clone._recurse = true;
        new_clone
    }
}
impl Parse for ParserStatAssVar {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssVar::default();
        if !self._recurse {
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
            if opts.nodes.len() > 0 {
                node.set_options(Some(opts.nodes.clone()));
            }

            // match space after "var" or after "[opts]"
            lex.expect_void()?;
            lex.jump(0, false)?;

            // march "(" to go recursively
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundO_) {
                let mut nodes: Nodes = List::new();

                // eat "("
                lex.jump(0, true)?; lex.eat();

                while !lex.curr(true)?.key().is_eof() {
                    // clone self and set recursive flag
                    let mut newself = self.recurse();
                    newself._oldstat = node.clone();
                    newself.parse(lex)?;
                    nodes.extend(newself.nodes);

                    //go to next one
                    lex.expect_terminal()?;
                    lex.bump();

                    // match and eat ")"
                    if matches!(lex.curr(true)?.key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                        lex.jump(0, true)?;
                        //expect endline
                        lex.expect_terminal()?;
                        break
                    }
                }
                self.nodes.extend(nodes);
                return Ok(());
            }
        } else {
            node = self._oldstat.clone();
        }

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(self._source.clone());
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(self._source.clone());
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            dt.parse(lex)?;
        }

        lex.expect_many(vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::symbol(SYMBOL::equal_),
            KEYWORD::void(VOID::endline_)
        ], true)?;

        if dt.nodes.len() > idents.nodes.len() {
            return Err( catch!( Typo::ParserTypeDisbalance {
                msg: Some(format!(
                    "number of identifiers: [{}] is smaller than number of types [{}]",
                    idents.nodes.len(),
                    dt.nodes.len(),
                    )),
                loc: Some(loc.clone()), 
                src: self._source.clone(),
            }))
        }

        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                node.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            node.set_ident(Some(idents.nodes.get(i).clone()));
            let mut newnode = Node::new(Box::new(node.clone()));
            newnode.set_loc(loc.clone());
            self.nodes.push(newnode);
        }
        lex.until_term(false)?;
        Ok(())
    }
}

impl ParserStatAssVar {
        pub fn extend(&mut self, n: Nodes) {
            self.nodes.extend(n)
        }
}
