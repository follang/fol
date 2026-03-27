use fol_runtime::std as rt;
use fol_runtime::std as rt_model;

pub(crate) const NAMESPACE_NAME: &str = "mixed_models_workspace::src::alloc";
pub(crate) const SOURCE_UNIT_IDS: &[usize] = &[1];

pub(crate) fn namespace_runtime_marker() -> &'static str {
    let _ = rt::crate_name();
    let _ = rt_model::tier_name();
    NAMESPACE_NAME
}

pub fn r__pkg__entry__mixed_models_workspace__r0__alloc_total() -> rt::FolInt {
    let mut l__pkg__entry__mixed_models_workspace__r0__l0__values: rt_model::FolSeq<rt::FolInt> = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r0__l1__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r0__l2__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r0__l3__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r0__l4__tmp: rt_model::FolSeq<rt::FolInt> = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r0__l5__tmp: rt_model::FolSeq<rt::FolInt> = Default::default();
    let mut l__pkg__entry__mixed_models_workspace__r0__l6__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__mixed_models_workspace__r0__l1__tmp = 1_i64;
                let l__pkg__entry__mixed_models_workspace__r0__l2__tmp = 2_i64;
                let l__pkg__entry__mixed_models_workspace__r0__l3__tmp = 3_i64;
                let l__pkg__entry__mixed_models_workspace__r0__l4__tmp = rt_model::FolSeq::from_items(vec![l__pkg__entry__mixed_models_workspace__r0__l1__tmp.clone(), l__pkg__entry__mixed_models_workspace__r0__l2__tmp.clone(), l__pkg__entry__mixed_models_workspace__r0__l3__tmp.clone()]);
                l__pkg__entry__mixed_models_workspace__r0__l0__values = l__pkg__entry__mixed_models_workspace__r0__l4__tmp.clone();
                let l__pkg__entry__mixed_models_workspace__r0__l5__tmp = l__pkg__entry__mixed_models_workspace__r0__l0__values.clone();
                let l__pkg__entry__mixed_models_workspace__r0__l6__tmp = rt::len(&l__pkg__entry__mixed_models_workspace__r0__l5__tmp);
                return l__pkg__entry__mixed_models_workspace__r0__l6__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}
