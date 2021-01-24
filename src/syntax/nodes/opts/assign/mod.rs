use std::fmt;
use crate::syntax::nodes::Tree;

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

impl fmt::Display for AssOpts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            AssOpts::imu => write!(f, "imu"),
            AssOpts::r#mut => write!(f, "mut"),
            AssOpts::sta => write!(f, "sta"),
            AssOpts::nor => write!(f, "nor"),
            AssOpts::exp => write!(f, "exp"),
            AssOpts::hid => write!(f, "hid"),
            AssOpts::stk => write!(f, "stk"),
            AssOpts::hep => write!(f, "hep"),
            AssOpts::ext => write!(f, "ext"),
        }
    }
}


