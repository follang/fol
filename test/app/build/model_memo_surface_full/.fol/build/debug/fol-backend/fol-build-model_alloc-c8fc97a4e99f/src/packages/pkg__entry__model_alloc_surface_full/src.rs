use fol_runtime::alloc as rt;
use fol_runtime::alloc as rt_model;

pub(crate) const NAMESPACE_NAME: &str = "model_alloc_surface_full::src";
pub(crate) const SOURCE_UNIT_IDS: &[usize] = &[1];

pub(crate) fn namespace_runtime_marker() -> &'static str {
    let _ = rt::crate_name();
    let _ = rt_model::tier_name();
    NAMESPACE_NAME
}

pub fn r__pkg__entry__model_alloc_surface_full__r0__join(l__pkg__entry__model_alloc_surface_full__r0__l0__prefix: rt_model::FolStr, l__pkg__entry__model_alloc_surface_full__r0__l1__extras: rt_model::FolSeq<rt_model::FolStr>) -> rt_model::FolStr {
    let mut l__pkg__entry__model_alloc_surface_full__r0__l2__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r0__l3__tmp: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r0__l4__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r0__l5__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r0__l6__tmp: rt_model::FolStr = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_alloc_surface_full__r0__l2__tmp = l__pkg__entry__model_alloc_surface_full__r0__l0__prefix.clone();
                let l__pkg__entry__model_alloc_surface_full__r0__l3__tmp = l__pkg__entry__model_alloc_surface_full__r0__l1__extras.clone();
                let l__pkg__entry__model_alloc_surface_full__r0__l4__tmp = 0_i64;
                let l__pkg__entry__model_alloc_surface_full__r0__l5__tmp = rt::index_seq(&l__pkg__entry__model_alloc_surface_full__r0__l3__tmp, l__pkg__entry__model_alloc_surface_full__r0__l4__tmp.clone()).unwrap().clone();
                let l__pkg__entry__model_alloc_surface_full__r0__l6__tmp = l__pkg__entry__model_alloc_surface_full__r0__l2__tmp + l__pkg__entry__model_alloc_surface_full__r0__l5__tmp;
                return l__pkg__entry__model_alloc_surface_full__r0__l6__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}

pub fn r__pkg__entry__model_alloc_surface_full__r1__main() -> rt::FolInt {
    let mut l__pkg__entry__model_alloc_surface_full__r1__l0__values: rt_model::FolVec<rt::FolInt> = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l1__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l2__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l3__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l4__tmp: rt_model::FolVec<rt::FolInt> = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l5__extras: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l6__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l7__tmp: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l8__text: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l9__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l10__tmp: rt_model::FolSeq<rt_model::FolStr> = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l11__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l12__tmp: rt_model::FolVec<rt::FolInt> = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l13__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l14__tmp: rt_model::FolStr = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l15__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_alloc_surface_full__r1__l16__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_alloc_surface_full__r1__l1__tmp = 1_i64;
                let l__pkg__entry__model_alloc_surface_full__r1__l2__tmp = 2_i64;
                let l__pkg__entry__model_alloc_surface_full__r1__l3__tmp = 3_i64;
                let l__pkg__entry__model_alloc_surface_full__r1__l4__tmp = rt_model::FolVec::from_items(vec![l__pkg__entry__model_alloc_surface_full__r1__l1__tmp.clone(), l__pkg__entry__model_alloc_surface_full__r1__l2__tmp.clone(), l__pkg__entry__model_alloc_surface_full__r1__l3__tmp.clone()]);
                l__pkg__entry__model_alloc_surface_full__r1__l0__values = l__pkg__entry__model_alloc_surface_full__r1__l4__tmp.clone();
                let l__pkg__entry__model_alloc_surface_full__r1__l6__tmp = rt_model::FolStr::from("-ready");
                let l__pkg__entry__model_alloc_surface_full__r1__l7__tmp = rt_model::FolSeq::from_items(vec![l__pkg__entry__model_alloc_surface_full__r1__l6__tmp.clone()]);
                l__pkg__entry__model_alloc_surface_full__r1__l5__extras = l__pkg__entry__model_alloc_surface_full__r1__l7__tmp.clone();
                let l__pkg__entry__model_alloc_surface_full__r1__l9__tmp = rt_model::FolStr::from("alloc");
                let l__pkg__entry__model_alloc_surface_full__r1__l10__tmp = l__pkg__entry__model_alloc_surface_full__r1__l5__extras.clone();
                let l__pkg__entry__model_alloc_surface_full__r1__l11__tmp = crate::packages::pkg__entry__model_alloc_surface_full::src::r__pkg__entry__model_alloc_surface_full__r0__join(l__pkg__entry__model_alloc_surface_full__r1__l9__tmp, l__pkg__entry__model_alloc_surface_full__r1__l10__tmp);
                l__pkg__entry__model_alloc_surface_full__r1__l8__text = l__pkg__entry__model_alloc_surface_full__r1__l11__tmp.clone();
                let l__pkg__entry__model_alloc_surface_full__r1__l12__tmp = l__pkg__entry__model_alloc_surface_full__r1__l0__values.clone();
                let l__pkg__entry__model_alloc_surface_full__r1__l13__tmp = rt::len(&l__pkg__entry__model_alloc_surface_full__r1__l12__tmp);
                let l__pkg__entry__model_alloc_surface_full__r1__l14__tmp = l__pkg__entry__model_alloc_surface_full__r1__l8__text.clone();
                let l__pkg__entry__model_alloc_surface_full__r1__l15__tmp = rt::len(&l__pkg__entry__model_alloc_surface_full__r1__l14__tmp);
                let l__pkg__entry__model_alloc_surface_full__r1__l16__tmp = l__pkg__entry__model_alloc_surface_full__r1__l13__tmp + l__pkg__entry__model_alloc_surface_full__r1__l15__tmp;
                return l__pkg__entry__model_alloc_surface_full__r1__l16__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}
