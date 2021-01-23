// pub mod binary;
use crate::syntax::ast::*;

#[derive(Clone, Debug)]
pub enum stat_type {
    Illegal,
    Use,
    Def,
    Var(var_stat),
    // Fun(fun_stat),
    Typ(typ_stat),
    Ali(typ_stat),
    Opts(assign_opts),
    Ident(String),
    Retype(retype_stat),
    If,
    When,
    Loop,
}


#[derive(Clone, Debug)]
pub struct var_stat {
    options: Option<Trees>,
    multi: Option<(usize, String)>,
    ident: Tree,
    retype: Option<Tree>,
    body: Option<Tree>,
}

impl var_stat {
    pub fn init() -> Self {
        var_stat {
            options: None,
            ident: Tree::new(
                point::Location::default(),
                tree_type::stat(stat_type::Ident(String::new())),
            ),
            multi: None,
            retype: None,
            body: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct typ_stat {
    options: Option<Trees>,
    multi: Option<(usize, String)>,
    ident: Tree,
    generics: Option<Vec<(Tree, Tree)>>,
    contract: Option<Vec<Tree>>,
    retype: Option<Tree>,
    body: Option<Tree>,
}
impl typ_stat {
    pub fn init() -> Self {
        typ_stat {
            options: None,
            multi: None,
            ident: Tree::new(
                point::Location::default(),
                tree_type::stat(stat_type::Ident(String::new())),
            ),
            generics: None,
            contract: None,
            retype: None,
            body: None,
        }
    }
}


#[derive(Clone, Debug)]
pub enum assign_opts {
    Imu,
    Mut,
    Sta,
    Nor,
    Exp,
    Hid,
    Stk,
    Hep,
    Ext,
}

#[derive(Clone, Debug)]
pub enum retype_stat {
    Int,
    Flt,
    Chr,
    Bol,
    Arr,
    Vec,
    Seq,
    Mat,
    Set,
    Map,
    Axi,
    Tab,
    Str,
    Num,
    Ptr,
    Err,
    Opt,
    Nev,
    Uni,
    Any,
    Non,
    Nil,
    Rec,
    Ent,
    Blu,
    Std,
    Loc,
    Url,
    Blk,
    Rut,
    Pat,
    Gen,
}
