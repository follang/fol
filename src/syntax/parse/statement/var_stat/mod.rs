use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;


pub struct VarStatParser {
    pub nodes: Nodes,
}
impl std::default::Default for VarStatParser {
    fn default() -> Self { Self { nodes: Nodes::new() } }
}

impl Parse for VarStatParser {
    fn parse(&mut self, lex: &mut lexer::Elements) -> Vod {
        let varstat: VarStat = VarStat::default();

        lex.expect( KEYWORD::option(OPTION::mut_) , true)?;
        lex.bump();
        // lex.expect( KEYWORD::option(OPTION::mut_) , true)?;
        lex.expect( KEYWORD::assign(ASSIGN::var_) , true)?;
        lex.toend();



        self.nodes.push(Node::new(Box::new(varstat)));
        Ok(())
    }
}
