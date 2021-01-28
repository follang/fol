use crate::types::*;
use crate::syntax::nodes::*;
use crate::syntax::lexer;
use super::Parse;

pub mod var_stat;
pub use crate::syntax::parse::statement::var_stat::*;

pub struct StatParser {}
impl std::default::Default for StatParser {
    fn default() -> Self { Self{} }
}

impl Parse for StatParser {
    fn parse(&mut self, mut lexer: &mut lexer::Elements) -> Con<Nodes> {
        let parse_var = VarStatParser::default().parse(&mut lexer);
        if let Err(e) = parse_var.clone() {
            println!("{}", e)
        }
        parse_var
    }
}
