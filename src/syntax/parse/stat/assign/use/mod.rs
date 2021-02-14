use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssUse};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::check;

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatAssUse {
    nodes: Nodes,
    _source: Source,
    _recurse: bool,
    _oldstat: NodeStatAssUse,
}

impl ParserStatAssUse {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init(src: Source) -> Self {
        Self { 
            nodes: Nodes::new(), 
            _source: src,
            _recurse: false,
            _oldstat: NodeStatAssUse::default()
        } 
    }
    pub fn recurse(&self) -> Self {
        let mut new_clone = self.clone();
        new_clone._recurse = true;
        new_clone
    }
}
impl Parse for ParserStatAssUse {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssUse::default();
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

            // match "use"
            check::expect(lex, KEYWORD::assign(ASSIGN::use_) , true)?;
            lex.jump(0, false)?;

            // match options after var  -> "[opts]"
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) {
                opts.parse(lex)?;
            }
            if opts.nodes.len() > 0 {
                node.set_options(Some(opts.nodes.clone()));
            }

            // match space after "var" or after "[opts]"
            check::expect_void(lex)?;
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
                    check::expect_terminal(lex)?;
                    lex.bump();

                    // match and eat ")"
                    if matches!(lex.curr(true)?.key(), KEYWORD::symbol(SYMBOL::roundC_)) {
                        lex.jump(0, true)?;
                        //expect endline
                        check::expect_terminal(lex)?;
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
        idents.only_one();
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(self._source.clone(), true);
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            dt.parse(lex)?;
        }

        check::expect_many(lex, vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::symbol(SYMBOL::equal_),
            KEYWORD::void(VOID::endline_)
        ], true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &self._source )?;

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

impl ParserStatAssUse {
        pub fn extend(&mut self, n: Nodes) {
            self.nodes.extend(n)
        }
}
