use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::token::*;
use crate::syntax::lexer;
use super::Parse;

pub mod var_stat;
pub use crate::syntax::parse::statement::var_stat::*;

pub struct StatParser {}
impl std::default::Default for StatParser {
    fn default() -> Self { Self{} }
}

impl Parse for StatParser {
    fn parse(&mut self, mut lex: &mut lexer::Elements) -> Con<Nodes> {
        if matches!(lex.curr(false).key(), KEYWORD::assign(ASSIGN::var_))
            || (matches!(lex.curr(false).key(), KEYWORD::option(_))
                && matches!(lex.peek(0, false).key(), KEYWORD::assign(ASSIGN::var_)))
        {
            let parse_var = VarStatParser::default().parse(&mut lex);
            if let Err(e) = parse_var.clone() {
                println!("{}", e)
            }
            return parse_var
        };
        halt!();
    }
}
