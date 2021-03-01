use crate::types::*;
use crate::syntax::index::Source;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;
use crate::syntax::parse::{eater, check};

use crate::syntax::nodes::stat::datatype;
use crate::syntax::parse::stat::assign::opts;
pub mod rec;
pub mod rut;


pub struct ParserStatDatatypes {
    pub nodes: Nodes,
    _inparam: bool,
}

impl ParserStatDatatypes {
    pub fn init(inparam: bool) -> Self {
        Self { 
            nodes: Nodes::new(),
            _inparam: inparam,
        } 
    }
}
impl Parse for ParserStatDatatypes {
    fn nodes(&self) -> Nodes { self.nodes.clone() }
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {

        // eat ":"
        if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::colon_) {
            lex.jump(0, true)?; 
        } else {
            return Ok(())
        }

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
                    let mut node = datatype::NodeStatDatatypes::default();
                    check::expect_ident(lex, true)?;
                    lex.eat();
                    node.set_data(lex.curr(true)?.con().to_string());
                    lex.jump(0, false)?; 
                    if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) { 
                        let mut op = opts::ParserStatAssOpts::init(true); 
                        op.parse(lex)?; 
                        if op.nodes.len() > 0 { node.set_form(Some(op.nodes.clone())); }
                    }
                    if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarO_) { 
                        eater::until_bracket(lex)?
                    }
                    let id = Node::new(Box::new(node));
                    self.nodes.push(id);
                }
            }

            if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::equal_)
                || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::semi_)
                || lex.curr(true)?.key().is_eol()
                || ((lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::roundC_)
                    || lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::squarC_))
                    && self._inparam)
            {
                lex.eat();
                break
            } else if lex.curr(true)?.key() == KEYWORD::symbol(SYMBOL::comma_) {
                lex.jump(0, true)?; lex.eat();
            }
            lex.eat();
        }

        Ok(())
    }
}
