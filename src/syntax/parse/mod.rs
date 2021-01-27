use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::lexer;

pub mod statement;
pub use crate::syntax::parse::statement::*;
// pub mod expression;


pub trait Parse {
    fn parse(self, lexer: &mut lexer::Elements) -> Con<Tree>;
}

pub struct Parser {}
impl std::default::Default for Parser {
    fn default() -> Self { Self{} }
}

impl Parse for Parser {
    fn parse(self, mut lexer: &mut lexer::Elements) -> Con<Tree> {
        let parse_stat = StatParser::default().parse(&mut lexer);
        parse_stat
    }
}
