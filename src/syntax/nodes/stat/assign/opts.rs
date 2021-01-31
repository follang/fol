use std::fmt;
use crate::syntax::nodes::{NodeTrait, OptsTrait};
use crate::syntax::token::{KEYWORD, OPTION};

#[derive(Clone)]
pub enum AssOptsTrait {
    mut_,
    imu_,
    sta_,
    nor_,
    exp_,
    hid_,
    stk_,
    hep_,
    ext_,
}

impl NodeTrait for AssOptsTrait {}
impl OptsTrait for AssOptsTrait {}

impl fmt::Display for AssOptsTrait {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            AssOptsTrait::imu_ => write!(f, "imu"),
            AssOptsTrait::mut_ => write!(f, "mut"),
            AssOptsTrait::sta_ => write!(f, "sta"),
            AssOptsTrait::nor_ => write!(f, "nor"),
            AssOptsTrait::exp_ => write!(f, "exp"),
            AssOptsTrait::hid_ => write!(f, "hid"),
            AssOptsTrait::stk_ => write!(f, "stk"),
            AssOptsTrait::hep_ => write!(f, "hep"),
            AssOptsTrait::ext_ => write!(f, "ext"),
        }
    }
}

impl From<OPTION> for AssOptsTrait {
    fn from(key: OPTION) -> Self {
        match key {
            OPTION::mut_ => AssOptsTrait::mut_,
            OPTION::imu_ => AssOptsTrait::imu_,
            OPTION::sta_ => AssOptsTrait::sta_,
            OPTION::nor_ => AssOptsTrait::nor_,
            OPTION::exp_ => AssOptsTrait::exp_,
            OPTION::hid_ => AssOptsTrait::hid_,
            OPTION::stk_ => AssOptsTrait::stk_,
            OPTION::hep_ => AssOptsTrait::hep_,
            OPTION::ext_ => AssOptsTrait::ext_,
            _ => unreachable!()
        }
    }
}

