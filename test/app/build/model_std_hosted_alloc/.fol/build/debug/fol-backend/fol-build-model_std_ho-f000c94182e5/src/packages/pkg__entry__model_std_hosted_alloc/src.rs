use fol_runtime::std as rt;
use fol_runtime::std as rt_model;

pub(crate) const NAMESPACE_NAME: &str = "model_std_hosted_alloc::src";
pub(crate) const SOURCE_UNIT_IDS: &[usize] = &[1];

pub(crate) fn namespace_runtime_marker() -> &'static str {
    let _ = rt::crate_name();
    let _ = rt_model::tier_name();
    NAMESPACE_NAME
}

pub fn r__pkg__entry__model_std_hosted_alloc__r0__banner(l__pkg__entry__model_std_hosted_alloc__r0__l0__prefix: rt_model::FolStr, l__pkg__entry__model_std_hosted_alloc__r0__l1__extras: rt_model::FolSeq<rt_model::FolStr>) -> rt_model::FolStr {
    let mut l__pkg__entry__model_std_hosted_alloc__r0__l2__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r0__l3__tmp: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r0__l4__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r0__l5__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r0__l6__tmp: rt_model::FolStr = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_std_hosted_alloc__r0__l2__tmp = l__pkg__entry__model_std_hosted_alloc__r0__l0__prefix.clone();
                let l__pkg__entry__model_std_hosted_alloc__r0__l3__tmp = l__pkg__entry__model_std_hosted_alloc__r0__l1__extras.clone();
                let l__pkg__entry__model_std_hosted_alloc__r0__l4__tmp = 0_i64;
                let l__pkg__entry__model_std_hosted_alloc__r0__l5__tmp = rt::index_seq(&l__pkg__entry__model_std_hosted_alloc__r0__l3__tmp, l__pkg__entry__model_std_hosted_alloc__r0__l4__tmp.clone()).unwrap().clone();
                let l__pkg__entry__model_std_hosted_alloc__r0__l6__tmp = l__pkg__entry__model_std_hosted_alloc__r0__l2__tmp + l__pkg__entry__model_std_hosted_alloc__r0__l5__tmp;
                return l__pkg__entry__model_std_hosted_alloc__r0__l6__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}

pub fn r__pkg__entry__model_std_hosted_alloc__r1__main() -> rt::FolInt {
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l0__extras: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l1__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l2__tmp: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l3__shown: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l4__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l5__tmp: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l6__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l7__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_std_hosted_alloc__r1__l8__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_std_hosted_alloc__r1__l1__tmp = rt_model::FolStr::from("-ready");
                let l__pkg__entry__model_std_hosted_alloc__r1__l2__tmp = rt_model::FolSeq::from_items(vec![l__pkg__entry__model_std_hosted_alloc__r1__l1__tmp.clone()]);
                l__pkg__entry__model_std_hosted_alloc__r1__l0__extras = l__pkg__entry__model_std_hosted_alloc__r1__l2__tmp.clone();
                let l__pkg__entry__model_std_hosted_alloc__r1__l4__tmp = rt_model::FolStr::from("std");
                let l__pkg__entry__model_std_hosted_alloc__r1__l5__tmp = l__pkg__entry__model_std_hosted_alloc__r1__l0__extras.clone();
                let l__pkg__entry__model_std_hosted_alloc__r1__l6__tmp = crate::packages::pkg__entry__model_std_hosted_alloc::src::r__pkg__entry__model_std_hosted_alloc__r0__banner(l__pkg__entry__model_std_hosted_alloc__r1__l4__tmp, l__pkg__entry__model_std_hosted_alloc__r1__l5__tmp);
                rt::echo(l__pkg__entry__model_std_hosted_alloc__r1__l6__tmp.clone());
                l__pkg__entry__model_std_hosted_alloc__r1__l3__shown = l__pkg__entry__model_std_hosted_alloc__r1__l6__tmp.clone();
                let l__pkg__entry__model_std_hosted_alloc__r1__l7__tmp = l__pkg__entry__model_std_hosted_alloc__r1__l3__shown.clone();
                let l__pkg__entry__model_std_hosted_alloc__r1__l8__tmp = rt::len(&l__pkg__entry__model_std_hosted_alloc__r1__l7__tmp);
                return l__pkg__entry__model_std_hosted_alloc__r1__l8__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}
