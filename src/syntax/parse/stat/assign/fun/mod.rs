use crate::types::{Vod, Errors};
use crate::syntax::nodes::{Node, Nodes, NodeStatDecL};
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{check, eater, Body};

use crate::syntax::parse::stat::ParserStat;
use crate::syntax::parse::stat::assign::opts::*;
use crate::syntax::parse::stat::ident::*;
use crate::syntax::parse::stat::datatype::*;
use crate::syntax::parse::stat::generics::*;
use crate::syntax::parse::stat::parameters::*;

#[derive(Clone)]
pub struct ParserStatAssFun {
    pub nodes: Nodes,
    pub errors: Errors,
}

impl ParserStatAssFun {
    pub fn len(&self) -> usize { self.nodes.len() }
        pub fn init() -> Self {
        Self {
            nodes: Nodes::new(),
            errors: Vec::new()
        } 
    }
    pub fn extend(&mut self, parser: &dyn Parse) { 
        self.nodes.extend(parser.nodes());
        self.errors.extend(parser.errors());
    }
}
impl Parse for ParserStatAssFun {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { Vec::new() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let loc = lex.curr(true)?.loc().clone();
        let mut node = NodeStatDecL::default();
        // match symbol before var  -> "~"
        let mut opts = ParserStatAssOpts::init();
        opts.parse(lex)?;

        // add "fun"
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

        // match indentifier "ident"
        let mut idents = ParserStatIdent::init();
        idents.once();
        idents.parse(lex)?; lex.eat();
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
        // lex.jump(0, true)?;

        eater::expr_body3(lex)?;

        // // match indentifier "body"
        // let mut body = ParserStat::init();
        // body.style(Body::Fun);
        // // if let Err(err) = body.parse(lex) { self.errors.push(err) }
        // let deep = lex.curr(false)?.loc().deep() -1;
        // println!("deep:{}", deep);
        // if let Err(err) = body.parse(lex) { 
        //     println!("{}", lex.curr(false)?.loc().deep());
        //     eater::stat_body(lex, deep)?; return Err(err) 
        // }
        // check::needs_body(lex, &body)?;
        // self.extend(&body);
        // if body.nodes.len() > 0 { node.set_body(Some(body.nodes)); }

        // check::expect(lex, KEYWORD::Symbol(SYMBOL::CurlyC), true)?;
        // lex.jump(0, true)?;

        let mut id = Node::new(Box::new(node.clone()));
        id.set_loc(loc.clone());
        self.nodes.push(id);

        Ok(())
    }
}
