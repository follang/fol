use crate::syntax::ast::Ast;


#[derive(Clone, Debug)]
pub enum AssOpts {
    imu,
    r#mut,
    sta,
    nor,
    exp,
    hid,
    stk,
    hep,
    ext,
}

impl Ast for AssOpts {}
