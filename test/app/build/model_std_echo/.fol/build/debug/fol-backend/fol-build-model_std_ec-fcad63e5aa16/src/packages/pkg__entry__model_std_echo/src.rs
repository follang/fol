use fol_runtime::prelude as rt;
use fol_runtime::std as rt_model;

pub(crate) const NAMESPACE_NAME: &str = "model_std_echo::src";
pub(crate) const SOURCE_UNIT_IDS: &[usize] = &[1];

pub(crate) fn namespace_runtime_marker() -> &'static str {
    let _ = rt::crate_name();
    let _ = rt_model::tier_name();
    NAMESPACE_NAME
}

pub fn r__pkg__entry__model_std_echo__r0__main() -> rt::FolInt {
    let mut l__pkg__entry__model_std_echo__r0__l0__shown: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_echo__r0__l1__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_echo__r0__l2__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_echo__r0__l3__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_std_echo__r0__l4__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_echo__r0__l5__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_std_echo__r0__l6__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_std_echo__r0__l1__tmp = rt_model::FolStr::from("std-ready");
                rt::echo(l__pkg__entry__model_std_echo__r0__l1__tmp.clone());
                l__pkg__entry__model_std_echo__r0__l0__shown = l__pkg__entry__model_std_echo__r0__l1__tmp.clone();
                let l__pkg__entry__model_std_echo__r0__l2__tmp = l__pkg__entry__model_std_echo__r0__l0__shown.clone();
                let l__pkg__entry__model_std_echo__r0__l3__tmp = rt::len(&l__pkg__entry__model_std_echo__r0__l2__tmp);
                let l__pkg__entry__model_std_echo__r0__l4__tmp = l__pkg__entry__model_std_echo__r0__l0__shown.clone();
                let l__pkg__entry__model_std_echo__r0__l5__tmp = rt::len(&l__pkg__entry__model_std_echo__r0__l4__tmp);
                let l__pkg__entry__model_std_echo__r0__l6__tmp = l__pkg__entry__model_std_echo__r0__l3__tmp - l__pkg__entry__model_std_echo__r0__l5__tmp;
                return l__pkg__entry__model_std_echo__r0__l6__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}
