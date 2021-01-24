use crate::syntax::ast::*;
use crate::syntax::ast::stat::*;

#[derive(Clone, Debug)]
pub struct TypStat {
    options: Option<Trees>,
    multi: Option<(usize, String)>,
    ident: Tree,
    generics: Option<Vec<(Tree, Tree)>>,
    contract: Option<Vec<Tree>>,
    retype: Option<Tree>,
    body: Option<Tree>,
}
impl TypStat {
    pub fn init() -> Self {
        Self {
            options: None,
            multi: None,
            ident: Tree::new(
                point::Location::default(),
                tree_type::stat(Stat::ident(String::new())),
            ),
            generics: None,
            contract: None,
            retype: None,
            body: None,
        }
    }
}

impl Ast for TypStat {}
