use crate::syntax::ast::Ast;


#[derive(Clone, Debug)]
pub enum TypOpts {
    int,
    flt,
    chr,
    bol,
    arr,
    vec,
    seq,
    mat,
    set,
    map,
    axi,
    tab,
    r#str,
    num,
    ptr,
    err,
    opt,
    nev,
    uni,
    any,
    non,
    nil,
    rec,
    ent,
    blu,
    std,
    loc,
    url,
    blk,
    rut,
    pat,
    gen,
}

impl Ast for TypOpts {}
