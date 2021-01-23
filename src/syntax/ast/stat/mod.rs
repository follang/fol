// pub mod binary;
use crate::syntax::ast::*;

#[derive(Clone, Debug)]
pub enum Stat {
    illegal,
    r#use,
    def,
    var(Var),
    // Fun(fun_stat),
    typ(Typ),
    ali(Typ),
    opts(assign_opts),
    ident(String),
    retype(retype_stat),
    r#if,
    when,
    r#loop,
}


#[derive(Clone, Debug)]
pub struct Var{
    options: Option<Trees>,
    multi: Option<(usize, String)>,
    ident: Tree,
    retype: Option<Tree>,
    body: Option<Tree>,
}

impl Var {
    pub fn init() -> Self {
        Var {
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

#[derive(Clone, Debug)]
pub struct Typ {
    options: Option<Trees>,
    multi: Option<(usize, String)>,
    ident: Tree,
    generics: Option<Vec<(Tree, Tree)>>,
    contract: Option<Vec<Tree>>,
    retype: Option<Tree>,
    body: Option<Tree>,
}
impl Typ {
    pub fn init() -> Self {
        Typ {
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
