use crate::syntax::ast::Tree;


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

impl Tree for AssOpts {}
