use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{eater, check};

pub mod rec;
pub mod rut;
pub mod r#box;


pub struct ParserStatDatatypes {
    pub nodes: Nodes,
    _colon: bool,
    _self: bool,
    _once: bool,
}

impl ParserStatDatatypes {
    pub fn init() -> Self {
        Self { 
            nodes: Nodes::new(),
            _colon: true,
            _self: false,
            _once: false,
        } 
    }
    pub fn nocolon(&mut self) { self._colon = false; }
    pub fn letself(&mut self) { self._self = true; }
    pub fn once(&mut self) { self._once = true; }
}
impl Parse for ParserStatDatatypes {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn errors(&self) -> Errors { Vec::new() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {

        // eat ":"
        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::Colon) || self._colon == false {
            lex.jump(0, true)?; 
        } else if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarO) && self._colon == false {
            lex.jump(0, true)?; 
        } else {
            return Ok(())
        }
        if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarC) && self._colon == false { return Ok(()) }

        while !lex.curr(true)?.key().is_eof() {
            match lex.curr(true)?.con().as_str() {
                "rec" => { 
                    let mut data = rec::ParserStatData::init(); 
                    data.parse(lex)?;
                    self.nodes.push(data.nodes().get(0));
                },
                "rut" => { 
                    let mut data = rut::ParserStatData::init(); 
                    data.parse(lex)?;
                    self.nodes.push(data.nodes().get(0));
                }
                _ => {
                    let mut node = NodeStatDatatypes::default();
                    if !(self._self && lex.curr(true)?.con().as_str() == "self") {
                        check::expect_ident_literal(lex, true)?;
                    }
                    lex.eat();
                    node.set_string(lex.curr(true)?.con().to_string());
                    lex.jump(0, false)?; 
                    if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarO) { 
                        let mut op = ParserStatDatatypes::init();
                        op.nocolon();
                        op.parse(lex)?; 
                        if op.nodes.len() > 0 { node.set_form(Some(op.nodes.clone())); }
                    //eat "]"
                    check::expect(lex, KEYWORD::Symbol(SYMBOL::SquarC), true)?;
                    lex.jump(0, false)?; 

                    }

                    if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::SquarO) { 
                        eater::until_bracket(lex)?
                    }
                    let id = Node::new(Box::new(node));
                    self.nodes.push(id);
                }
            }
            if self._once { lex.eat(); break }
            if lex.curr(true)?.key() == KEYWORD::Symbol(SYMBOL::Comma) {
                lex.jump(0, true)?; lex.eat();
                lex.eat();
            } else {
                lex.eat();
                break
            }
        }

        Ok(())
    }
}
