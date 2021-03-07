use crate::types::{Vod, List, Errors};
use crate::syntax::nodes::{Node, Nodes, NodeStatDecL};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, Body};

use crate::syntax::parse::stat::ParserStat;
use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::parameters::*;
use crate::syntax::parse::stat::generics::*;
use crate::syntax::parse::stat::datatype::*;

#[derive(Clone)]
pub struct ParserStatAssTyp {
    nodes: Nodes,
    errors: Errors,
    _level: usize,
    _recurse: bool,
    _oldstat: NodeStatDecL,
}

impl ParserStatAssTyp {
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn init(level: usize) -> Self {
        Self {
            nodes: Nodes::new(),
            errors: Vec::new(),
            _recurse: false,
            _oldstat: NodeStatDecL::default(),
            _level: level,
        } 
    }
    pub fn level(&self) -> usize { self._level }
}
impl Parse for ParserStatAssTyp {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { self.errors.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let mut loc = lex.curr(true)?.loc().clone();
        loc.set_deep(self.level() as isize);
        let mut node = NodeStatDecL::default();
        if !self._recurse {
            // match symbol before var  -> "~"
            let mut opts = ParserStatAssOpts::init();
            opts.parse(lex)?;

            // add "typ"
            node.set_string(lex.curr(true)?.con().to_string());
            lex.jump(0, false)?;

            // match options after var  -> "[opts]"
            opts.parse(lex)?;
            if opts.nodes.len() > 0 { node.set_options(Some(opts.nodes.clone())); }

            // match generics
            let mut gen = ParserStatGenerics::init();
            gen.parse(lex)?;
            if gen.nodes.len() > 0 { node.set_generics(Some(gen.nodes.clone())); }
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
        idents.once();
        idents.parse(lex)?;
        node.set_ident(Some(idents.nodes.get(0).clone()));

        // match parameters after (  -> "(one, two)"
        let mut parameters = ParserStatParameters::init();
        parameters.parse(lex)?; lex.eat();
        if parameters.nodes.len() > 0 { node.set_parameters(Some(parameters.nodes.clone())) }

        // match datatypes after :  -> "int[opts][]"
        let mut dt = ParserStatDatatypes::init();
        dt.parse(lex)?;
        if dt.nodes.len() > 0 { node.set_datatype(Some(dt.nodes.get(0).clone())); }

        check::expect(lex, KEYWORD::Symbol(SYMBOL::Equal), true)?;
        lex.jump(0, true)?;
        check::expect(lex, KEYWORD::Symbol(SYMBOL::CurlyO), true)?;
        lex.jump(0, true)?;
 

        // match indentifier "body"
        let mut body = ParserStat::init(Body::Typ, self.level() + 1);
        if let Err(err) = body.parse(lex) { self.errors.push(err) }
        self.errors.extend(body.errors());
        check::needs_body(loc.clone(), lex, &body)?;
        node.set_body(Some(body.nodes()));

        // check::expect(lex, KEYWORD::Symbol(SYMBOL::CurlyC), true)?;
        if matches!(lex.curr(true)?.key(), KEYWORD::Symbol(SYMBOL::CurlyC)) { lex.jump(0, true)?; }

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        Ok(())
    }
}

impl ParserStatAssTyp {
    fn recurse(&mut self, node: &NodeStatDecL, lex: &mut lexer::Elements) -> Vod {
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
