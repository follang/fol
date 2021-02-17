use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssTyp};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater};

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::contracts::*;
use crate::syntax::parse::stat::datatype::*;

#[derive(Clone)]
pub struct ParserStatAssTyp {
    pub nodes: Nodes,
    _recurse: bool,
    _oldstat: NodeStatAssTyp,
}

impl ParserStatAssTyp {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self { nodes: Nodes::new(), _recurse: false, _oldstat: NodeStatAssTyp::default() } 
    }
    pub fn recurse(&self) -> Self {
        let mut new_clone = self.clone();
        new_clone._recurse = true;
        new_clone
    }
}
impl Parse for ParserStatAssTyp {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssTyp::default();
        if !self._recurse {
            // match symbol before var  -> "~"
            let mut opts = ParserStatAssOpts::init(false);
            opts.parse(lex)?;

            // match "var"
            check::expect(lex,  KEYWORD::assign(ASSIGN::typ_) , true)?;
            lex.jump(0, false)?;

            // match options after var  -> "[opts]"
            opts.parse(lex)?;
            if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }

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
                    lex.jump(0, false)?;

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
        let mut idents = ParserStatIdent::init(true);
        idents.parse(lex)?;
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // match contracts after (  -> "(one, two)"
        let mut contracts = ParserStatContract::init();
        contracts.parse(lex)?;
        if contracts.nodes.len() > 0 { node.set_contracts(Some(contracts.nodes.clone())) }

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(true);
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect(lex, KEYWORD::symbol(SYMBOL::equal_), true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        eater::until_term(lex, false)?;
        lex.debug(false).ok();
        Ok(())
    }
}

impl ParserStatAssTyp {
        pub fn extend(&mut self, n: Nodes) {
            self.nodes.extend(n)
        }
}
