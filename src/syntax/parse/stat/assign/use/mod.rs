use crate::types::{Vod, List, error::*};
use crate::syntax::index::Source;
use crate::syntax::nodes::{Node, Nodes, NodeStatAssUse};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater};

use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatAssUse {
    nodes: Nodes,
    _recurse: bool,
    _oldstat: NodeStatAssUse,
}

impl ParserStatAssUse {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self { 
            nodes: Nodes::new(), 
            _recurse: false,
            _oldstat: NodeStatAssUse::default()
        } 
    }
}
impl Parse for ParserStatAssUse {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatAssUse::default();
        if !self._recurse {
            // match symbol before var  -> "~"
            let mut opts = ParserStatAssOpts::init(false);
            opts.parse(lex)?;

            // match "use"
            check::expect(lex, KEYWORD::assign(ASSIGN::use_) , true)?;
            node.set_string(lex.curr(true)?.con().to_string());
            lex.jump(0, false)?;

            // match options after var  -> "[opts]"
            opts.parse(lex)?;
            if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }
            check::expect_void(lex)?;

            // march "(" to go recursively
            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundO_) {
                self.recurse(&node, lex)?;
                return Ok(());
            }
        } else {
            node = self._oldstat.clone();
        }

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init(true);
        idents.parse(lex)?;
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init(true);
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect_many(lex, vec![ 
            KEYWORD::symbol(SYMBOL::semi_),
            KEYWORD::symbol(SYMBOL::equal_),
            KEYWORD::void(VOID::endline_)
        ], true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        eater::until_term(lex, false)?;
        Ok(())
    }
}

impl ParserStatAssUse {
    fn recurse(&mut self, node: &NodeStatAssUse, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundO_) {
            lex.jump(0, true)?; lex.eat();

            let mut nodes: Nodes = List::new();
            while !lex.curr(true)?.key().is_eof() {
                // clone self and set recursive flag
                let mut newself = self.clone();
                newself._recurse = true;
                newself._oldstat = node.clone();
                newself.parse(lex)?;
                nodes.extend(newself.nodes);

                //go to next one
                check::expect_terminal(lex, )?;
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
        }
        return Ok(())
    }
}
