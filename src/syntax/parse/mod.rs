use crate::syntax::lexer;
use crate::syntax::nodes::*;
use crate::types::Con;

pub trait Parser: Sized {
    type Output;
    fn parse(self, lexer: &mut lexer::Elements) -> Con<Self::Output>;
}
