use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;


pub struct VarStatParser {
    multi: bool,
}
impl std::default::Default for VarStatParser {
    fn default() -> Self { Self { multi: false } }
}

impl Parse for VarStatParser {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Con<Nodes> {
        lex.expect( KEYWORD::option(OPTION::mut_) , true)?;
        lex.bump();
        // lex.expect( KEYWORD::option(OPTION::mut_) , true)?;
        lex.expect( KEYWORD::assign(ASSIGN::var_) , true)?;
        let nodes: Nodes = Vec::new();
        for e in lex {
            println!("{}", e);
        }
        Ok(nodes)
    }
}
