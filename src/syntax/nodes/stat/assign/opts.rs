use std::fmt;
use crate::syntax::nodes::{NodeTrait, OptsTrait};

#[derive(Clone)]
pub enum AssOptsTrait {
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

impl NodeTrait for AssOptsTrait {}
impl OptsTrait for AssOptsTrait {}

impl fmt::Display for AssOptsTrait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            AssOptsTrait::imu => write!(f, "imu"),
            AssOptsTrait::r#mut => write!(f, "mut"),
            AssOptsTrait::sta => write!(f, "sta"),
            AssOptsTrait::nor => write!(f, "nor"),
            AssOptsTrait::exp => write!(f, "exp"),
            AssOptsTrait::hid => write!(f, "hid"),
            AssOptsTrait::stk => write!(f, "stk"),
            AssOptsTrait::hep => write!(f, "hep"),
            AssOptsTrait::ext => write!(f, "ext"),
        }
    }
}


