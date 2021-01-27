use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::lexer;
use super::Parse;


pub struct VarStatParser {}
impl std::default::Default for VarStatParser {
    fn default() -> Self { Self{} }
}

impl Parse for VarStatParser {
    fn parse(self, lexer: &mut lexer::Elements) -> Con<Tree> {
        todo!();
    }
}
