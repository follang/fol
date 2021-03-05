use crate::types::{Vod, List};
use crate::syntax::nodes::{Node, Nodes, NodeStatDecS};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

use crate::syntax::parse::check;
use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;


#[derive(Clone)]
pub struct ParserStatAssVar {
    pub nodes: Nodes,
    _recurse: bool,
    _oldstat: NodeStatDecS,
}

impl ParserStatAssVar {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init() -> Self {
        Self { nodes: Nodes::new(), _recurse: false, _oldstat: NodeStatDecS::default() } 
    }
}
impl Parse for ParserStatAssVar {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatDecS::default();
        if !self._recurse {
            // match symbol before var  -> "~"
            let mut opts = ParserStatAssOpts::init();
            opts.parse(lex)?;

            // add "var"
            node.set_string(lex.curr(true)?.con().to_string());
            lex.jump(0, false)?;

            // match options after var  -> "[opts]"
            opts.parse(lex)?;
            if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }
            check::expect_void(lex)?;

            // march "(" to go recursively
            if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::RoundO) {
                self.recurse(&node, lex)?;
                return Ok(());
            }
        } else {
            node = self._oldstat.clone();
        }

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init();
        idents.parse(lex)?; lex.eat();

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init();
        dt.parse(lex)?;

        check::expect_many(lex, vec![ 
            KEYWORD::Symbol(SYMBOL::Semi),
            KEYWORD::Symbol(SYMBOL::Equal),
            KEYWORD::Void(VOID::EndLine)
        ], true)?;
        check::type_balance(idents.nodes.len(), dt.nodes.len(), &loc, &lex.curr(false)?.loc().source() )?;

        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::Semi) || lex.curr(true)?.key() == KEYWORD::Void(VOID::EndLine) {
            for i in 0..idents.nodes.len() {
                if dt.nodes.len() > 0 {
                    let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                    node.set_datatype(Some(dt.nodes.get(idx).clone()));
                }
                node.set_ident(Some(idents.nodes.get(i).clone()));
                let mut id = Node::new(Box::new(node.clone()));
                id.set_loc(loc.clone());
                self.nodes.push(id);
            }
            return Ok(())
        }

        check::expect(lex, KEYWORD::Symbol(SYMBOL::Equal), true)?;

        for i in 0..idents.nodes.len() {
            if dt.nodes.len() > 0 {
                let idx = if i >= dt.nodes.len() { dt.nodes.len()-1 } else { i };
                node.set_datatype(Some(dt.nodes.get(idx).clone()));
            }
            node.set_ident(Some(idents.nodes.get(i).clone()));
            let mut id = Node::new(Box::new(node.clone()));
            id.set_loc(loc.clone());
            self.nodes.push(id);
        }

        lex.until_term(false)?;
        Ok(())
    }
}

impl ParserStatAssVar {
    fn recurse(&mut self, node: &NodeStatDecS, lex: &mut lexer::Elements) -> Vod {
        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::RoundO) {
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
                if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::RoundC)) {
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
