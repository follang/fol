use fol_runtime::core as rt;
use fol_runtime::core as rt_model;

pub(crate) const NAMESPACE_NAME: &str = "model_core_surface_full::src";
pub(crate) const SOURCE_UNIT_IDS: &[usize] = &[1];

pub(crate) fn namespace_runtime_marker() -> &'static str {
    let _ = rt::crate_name();
    let _ = rt_model::tier_name();
    NAMESPACE_NAME
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ty__pkg__entry__model_core_surface_full__t6__status {
    FAIL(rt::FolInt),
    OK(rt::FolInt),
}

impl Default for ty__pkg__entry__model_core_surface_full__t6__status {
    fn default() -> Self {
        Self::FAIL(Default::default())
    }
}

impl rt::FolEntry for ty__pkg__entry__model_core_surface_full__t6__status {
    fn fol_entry_name(&self) -> &'static str {
        "Status"
    }

    fn fol_entry_variant_name(&self) -> &'static str {
        match self {
            Self::FAIL(..) => "FAIL",
            Self::OK(..) => "OK",
        }
    }

    fn fol_entry_fields(&self) -> Vec<rt::FolNamedValue> {
        match self {
            Self::FAIL(payload) => vec![rt::FolNamedValue::new("payload", payload.to_string())],
            Self::OK(payload) => vec![rt::FolNamedValue::new("payload", payload.to_string())],
        }
    }
}

impl rt::FolEchoFormat for ty__pkg__entry__model_core_surface_full__t6__status {
    fn fol_echo_format(&self) -> String {
        rt::render_entry(self)
    }
}

impl std::fmt::Display for ty__pkg__entry__model_core_surface_full__t6__status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", rt::render_entry(self))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ty__pkg__entry__model_core_surface_full__t7__counter {
    pub step: rt::FolInt,
    pub total: rt::FolInt,
}

impl rt::FolRecord for ty__pkg__entry__model_core_surface_full__t7__counter {
    fn fol_record_name(&self) -> &'static str {
        "Counter"
    }

    fn fol_record_fields(&self) -> Vec<rt::FolNamedValue> {
        vec![
            rt::FolNamedValue::new("step", self.step.to_string()),
            rt::FolNamedValue::new("total", self.total.to_string()),
        ]
    }
}

impl rt::FolEchoFormat for ty__pkg__entry__model_core_surface_full__t7__counter {
    fn fol_echo_format(&self) -> String {
        rt::render_record(self)
    }
}

impl std::fmt::Display for ty__pkg__entry__model_core_surface_full__t7__counter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", rt::render_record(self))
    }
}

pub fn r__pkg__entry__model_core_surface_full__r0__read(receiver: crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t7__counter, l__pkg__entry__model_core_surface_full__r0__l1__by: rt::FolInt) -> rt::FolInt {
    let mut l__pkg__entry__model_core_surface_full__r0__l2__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r0__l3__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r0__l4__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_core_surface_full__r0__l2__tmp = l__pkg__entry__model_core_surface_full__r0__l1__by.clone();
                let l__pkg__entry__model_core_surface_full__r0__l3__tmp = 2_i64;
                let l__pkg__entry__model_core_surface_full__r0__l4__tmp = l__pkg__entry__model_core_surface_full__r0__l2__tmp + l__pkg__entry__model_core_surface_full__r0__l3__tmp;
                return l__pkg__entry__model_core_surface_full__r0__l4__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}

pub fn r__pkg__entry__model_core_surface_full__r1__choose(l__pkg__entry__model_core_surface_full__r1__l0__active: rt::FolBool) -> rt::FolOption<rt::FolInt> {
    let mut l__pkg__entry__model_core_surface_full__r1__l1__tmp: rt::FolBool = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r1__l2__tmp: rt::FolBool = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r1__l3__tmp: rt::FolBool = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r1__l4__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r1__l5__tmp: rt::FolOption<rt::FolInt> = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r1__l6__tmp: rt::FolOption<rt::FolInt> = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_core_surface_full__r1__l1__tmp = l__pkg__entry__model_core_surface_full__r1__l0__active.clone();
                let l__pkg__entry__model_core_surface_full__r1__l2__tmp = true;
                let l__pkg__entry__model_core_surface_full__r1__l3__tmp = l__pkg__entry__model_core_surface_full__r1__l1__tmp == l__pkg__entry__model_core_surface_full__r1__l2__tmp;
                if l__pkg__entry__model_core_surface_full__r1__l3__tmp { __fol_next_block = 1; } else { __fol_next_block = 2; } continue;
            },
            1 => {
                let l__pkg__entry__model_core_surface_full__r1__l4__tmp = 7_i64;
                let l__pkg__entry__model_core_surface_full__r1__l5__tmp = rt::FolOption::some(l__pkg__entry__model_core_surface_full__r1__l4__tmp.clone());
                return l__pkg__entry__model_core_surface_full__r1__l5__tmp;
            },
            2 => {
                let l__pkg__entry__model_core_surface_full__r1__l6__tmp = rt::FolOption::nil();
                return l__pkg__entry__model_core_surface_full__r1__l6__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}

pub fn r__pkg__entry__model_core_surface_full__r2__main() -> rt::FolInt {
    let mut l__pkg__entry__model_core_surface_full__r2__l0__values: rt::FolArray<rt::FolInt, 2> = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l1__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l2__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l3__tmp: rt::FolArray<rt::FolInt, 2> = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l4__current: crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t7__counter = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l5__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l6__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l7__tmp: crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t7__counter = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l8__state: crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t6__status = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l9__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l10__tmp: crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t6__status = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l11__chosen: rt::FolOption<rt::FolInt> = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l12__tmp: rt::FolBool = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l13__tmp: rt::FolOption<rt::FolInt> = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l14__tmp: rt::FolArray<rt::FolInt, 2> = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l15__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l16__tmp: crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t7__counter = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l17__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l18__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l19__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l20__tmp: rt::FolOption<rt::FolInt> = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l21__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l22__tmp: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l23__done: rt::FolInt = Default::default();
    let mut l__pkg__entry__model_core_surface_full__r2__l24__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__model_core_surface_full__r2__l1__tmp = 3_i64;
                let l__pkg__entry__model_core_surface_full__r2__l2__tmp = 4_i64;
                let l__pkg__entry__model_core_surface_full__r2__l3__tmp = [l__pkg__entry__model_core_surface_full__r2__l1__tmp.clone(), l__pkg__entry__model_core_surface_full__r2__l2__tmp.clone()];
                l__pkg__entry__model_core_surface_full__r2__l0__values = l__pkg__entry__model_core_surface_full__r2__l3__tmp.clone();
                let l__pkg__entry__model_core_surface_full__r2__l5__tmp = 5_i64;
                let l__pkg__entry__model_core_surface_full__r2__l6__tmp = 2_i64;
                let l__pkg__entry__model_core_surface_full__r2__l7__tmp = crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t7__counter { total: l__pkg__entry__model_core_surface_full__r2__l5__tmp.clone(), step: l__pkg__entry__model_core_surface_full__r2__l6__tmp.clone() };
                l__pkg__entry__model_core_surface_full__r2__l4__current = l__pkg__entry__model_core_surface_full__r2__l7__tmp.clone();
                let l__pkg__entry__model_core_surface_full__r2__l9__tmp = 1_i64;
                let l__pkg__entry__model_core_surface_full__r2__l10__tmp = crate::packages::pkg__entry__model_core_surface_full::src::ty__pkg__entry__model_core_surface_full__t6__status::OK(l__pkg__entry__model_core_surface_full__r2__l9__tmp.clone());
                l__pkg__entry__model_core_surface_full__r2__l8__state = l__pkg__entry__model_core_surface_full__r2__l10__tmp.clone();
                let l__pkg__entry__model_core_surface_full__r2__l12__tmp = true;
                let l__pkg__entry__model_core_surface_full__r2__l13__tmp = crate::packages::pkg__entry__model_core_surface_full::src::r__pkg__entry__model_core_surface_full__r1__choose(l__pkg__entry__model_core_surface_full__r2__l12__tmp);
                l__pkg__entry__model_core_surface_full__r2__l11__chosen = l__pkg__entry__model_core_surface_full__r2__l13__tmp.clone();
                let l__pkg__entry__model_core_surface_full__r2__l14__tmp = l__pkg__entry__model_core_surface_full__r2__l0__values.clone();
                let l__pkg__entry__model_core_surface_full__r2__l15__tmp = rt::len(&l__pkg__entry__model_core_surface_full__r2__l14__tmp);
                let l__pkg__entry__model_core_surface_full__r2__l16__tmp = l__pkg__entry__model_core_surface_full__r2__l4__current.clone();
                let l__pkg__entry__model_core_surface_full__r2__l17__tmp = 1_i64;
                let l__pkg__entry__model_core_surface_full__r2__l18__tmp = crate::packages::pkg__entry__model_core_surface_full::src::r__pkg__entry__model_core_surface_full__r0__read(l__pkg__entry__model_core_surface_full__r2__l16__tmp, l__pkg__entry__model_core_surface_full__r2__l17__tmp);
                let l__pkg__entry__model_core_surface_full__r2__l19__tmp = l__pkg__entry__model_core_surface_full__r2__l15__tmp + l__pkg__entry__model_core_surface_full__r2__l18__tmp;
                let l__pkg__entry__model_core_surface_full__r2__l20__tmp = l__pkg__entry__model_core_surface_full__r2__l11__chosen.clone();
                let l__pkg__entry__model_core_surface_full__r2__l21__tmp = rt::unwrap_optional_shell(l__pkg__entry__model_core_surface_full__r2__l20__tmp.clone()).unwrap();
                let l__pkg__entry__model_core_surface_full__r2__l22__tmp = l__pkg__entry__model_core_surface_full__r2__l19__tmp + l__pkg__entry__model_core_surface_full__r2__l21__tmp;
                let l__pkg__entry__model_core_surface_full__r2__l24__tmp = 1_i64;
                l__pkg__entry__model_core_surface_full__r2__l23__done = l__pkg__entry__model_core_surface_full__r2__l24__tmp.clone();
                return l__pkg__entry__model_core_surface_full__r2__l22__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}
