use crate::syntax::ast::*;
use crate::syntax::ast::stat::*;

#[derive(Clone, Debug)]
pub struct VarStat{
    options: Option<Trees>,
    multi: Option<(usize, String)>,
    ident: Tree,
    retype: Option<Tree>,
    body: Option<Tree>,
}

impl VarStat {
    pub fn init() -> Self {
        Self {
            options: None,
            ident: Tree::new(
                point::Location::default(),
                tree_type::stat(Stat::ident(String::new())),
            ),
            multi: None,
            retype: None,
            body: None,
        }
    }
}


impl Ast for VarStat {}
