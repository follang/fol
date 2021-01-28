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
    fn parse(&mut self, lexer: &mut lexer::Elements) -> Con<Nodes> {
        // lexer.expect( KEYWORD::comment , true)?;
        let nodes: Nodes = Vec::new();
        for e in lexer {
            println!("{}", e);
        }
        Ok(nodes)
    }
}
