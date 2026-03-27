use fol_runtime::std as rt;
use fol_runtime::std as rt_model;

pub(crate) const NAMESPACE_NAME: &str = "mixed_models_workspace::src";
pub(crate) const SOURCE_UNIT_IDS: &[usize] = &[3];

pub(crate) fn namespace_runtime_marker() -> &'static str {
    let _ = rt::crate_name();
    let _ = rt_model::tier_name();
    NAMESPACE_NAME
}

pub fn r__pkg__entry__mixed_models_workspace__r2__main() -> rt::FolInt {
    let mut l__pkg__entry__mixed_models_workspace__r2__l0__tmp: rt::FolInt = Default::default();
    let mut __fol_next_block: usize = 0;
    loop {
        match __fol_next_block {
            0 => {
                let l__pkg__entry__mixed_models_workspace__r2__l0__tmp = 7_i64;
                rt::echo(l__pkg__entry__mixed_models_workspace__r2__l0__tmp.clone());
                return l__pkg__entry__mixed_models_workspace__r2__l0__tmp;
            },
            _ => unreachable!("invalid lowered block {}", __fol_next_block),
        }
    }
}
