use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::check;

pub use crate::syntax::nodes::stat::assign::opts::*;

pub struct ParserStatAssOpts {
    pub nodes: Nodes,
    _source: Source,
    pub recivers: Nodes,
    _recivers: bool,
}

impl ParserStatAssOpts {
    pub fn init(src: Source, recivers: bool) -> Self {
        Self { 
            nodes: Nodes::new(),
            _source: src,
            _recivers: recivers,
            recivers: Nodes::new(),
        } 
    }
}
impl Parse for ParserStatAssOpts {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        // eat "["
        lex.jump(0, false)?;

        // match "]" if there and return
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_) {
            lex.jump(0, true)?;
            return Ok(())
        }
        while !lex.curr(true)?.key().is_eof() {
            // checks if no recivers then expect only options
            if !self._recivers { check::expect_option(lex, true)?; }

            // checks if option
            if let KEYWORD::option(a) = lex.curr(true)?.key() {
                let assopt: AssOptsTrait = a.into();
                let node = Node::new(Box::new(assopt));
                self.nodes.push(node);
            // checks if option or type (those are recivers) -> only for procedures
            } else if lex.curr(true)?.key().is_ident() || lex.curr(true)?.key().is_type(){
                let reviver = NodeStatIdent::new(lex.curr(true)?.con().clone());
                let node = Node::new(Box::new(reviver));
                self.recivers.push(node);
            // error if not procedure, type or ident
            } else {
                check::expect_many(lex, vec![ 
                    KEYWORD::ident,
                    KEYWORD::option(OPTION::ANY),
                    KEYWORD::types(TYPE::ANY)
                ], true)?;
            }
            lex.jump(0, true)?;

            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_)
                || lex.curr(true)?.key().is_eol()
            {
                lex.jump(0, true)?;
                break
            } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::comma_) {
                if lex.peek(0, true)?.key() == KEYWORD::symbol(SYMBOL::squarC_) 
                {
                    lex.jump(1, true)?;
                    break
                }
                lex.jump(0, false)?;
            }
        }
        Ok(())
    }
}

impl ParserStatAssOpts {
    pub fn push(&mut self, node: Node) {
        self.nodes.push(node);
    }
}
