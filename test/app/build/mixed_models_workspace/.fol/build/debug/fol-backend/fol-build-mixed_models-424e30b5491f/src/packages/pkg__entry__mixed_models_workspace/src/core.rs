use fol_runtime::std as rt;
use fol_runtime::std as rt_model;

pub(crate) const NAMESPACE_NAME: &str = "mixed_models_workspace::src::core";
pub(crate) const SOURCE_UNIT_IDS: &[usize] = &[2];

pub(crate) fn namespace_runtime_marker() -> &'static str {
    let _ = rt::crate_name();
    let _ = rt_model::tier_name();
    NAMESPACE_NAME
}

pub fn r__pkg__entry__mixed_models_workspace__r1__core_total() -> rt::FolInt {
    let mut l__pkg__entry__mixed_models_workspace__r1__l0__values: rt::FolArray<rt::FolInt, 2> = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l1__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l2__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l3__tmp: rt::FolArray<rt::FolInt, 2> = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l4__tmp: rt::FolArray<rt::FolInt, 2> = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l5__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l6__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l7__tmp: rt::FolArray<rt::FolInt, 2> = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l8__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l9__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r1__l10__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__mixed_models_workspace__r1__l1__tmp = 1_i64;
                let l__pkg__entry__mixed_models_workspace__r1__l2__tmp = 2_i64;
                let l__pkg__entry__mixed_models_workspace__r1__l3__tmp = [l__pkg__entry__mixed_models_workspace__r1__l1__tmp.clone(), l__pkg__entry__mixed_models_workspace__r1__l2__tmp.clone()];
                l__pkg__entry__mixed_models_workspace__r1__l0__values = l__pkg__entry__mixed_models_workspace__r1__l3__tmp.clone();
                let l__pkg__entry__mixed_models_workspace__r1__l4__tmp = l__pkg__entry__mixed_models_workspace__r1__l0__values.clone();
                let l__pkg__entry__mixed_models_workspace__r1__l5__tmp = 0_i64;
                let l__pkg__entry__mixed_models_workspace__r1__l6__tmp = rt::index_array(&l__pkg__entry__mixed_models_workspace__r1__l4__tmp, l__pkg__entry__mixed_models_workspace__r1__l5__tmp.clone()).unwrap().clone();
                let l__pkg__entry__mixed_models_workspace__r1__l7__tmp = l__pkg__entry__mixed_models_workspace__r1__l0__values.clone();
                let l__pkg__entry__mixed_models_workspace__r1__l8__tmp = 1_i64;
                let l__pkg__entry__mixed_models_workspace__r1__l9__tmp = rt::index_array(&l__pkg__entry__mixed_models_workspace__r1__l7__tmp, l__pkg__entry__mixed_models_workspace__r1__l8__tmp.clone()).unwrap().clone();
                let l__pkg__entry__mixed_models_workspace__r1__l10__tmp = l__pkg__entry__mixed_models_workspace__r1__l6__tmp + l__pkg__entry__mixed_models_workspace__r1__l9__tmp;
                return l__pkg__entry__mixed_models_workspace__r1__l10__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}
