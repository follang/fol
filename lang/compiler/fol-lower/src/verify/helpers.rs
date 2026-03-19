use crate::{LoweredWorkspace, LoweringError, LoweringErrorKind};
use std::collections::VecDeque;

pub(super) fn verify_local_reference(
    routine: &crate::LoweredRoutine,
    instr_id: usize,
    label: &str,
    local: crate::LoweredLocalId,
    errors: &mut Vec<LoweringError>,
) {
    if routine.locals.get(local).is_none() {
        errors.push(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "lowered routine '{}' instruction {} uses missing {} local {}",
                routine.name, instr_id, label, local.0
            ),
        ));
    }
}

pub(super) fn verify_type_reference(
    workspace: &LoweredWorkspace,
    package: &crate::LoweredPackage,
    routine: &crate::LoweredRoutine,
    instr_id: usize,
    label: &str,
    type_id: crate::LoweredTypeId,
    errors: &mut Vec<LoweringError>,
) {
    if workspace.type_table().get(type_id).is_none() {
        errors.push(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "lowered package '{}' routine '{}' instruction {} uses missing {} {}",
                package.identity.display_name, routine.name, instr_id, label, type_id.0
            ),
        ));
    }
}

pub(super) fn enqueue_target(
    routine_name: &str,
    blocks: &crate::IdTable<crate::LoweredBlockId, crate::LoweredBlock>,
    errors: &mut Vec<LoweringError>,
    queue: &mut VecDeque<crate::LoweredBlockId>,
    target: crate::LoweredBlockId,
) {
    if blocks.get(target).is_none() {
        errors.push(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "lowered routine '{}' terminator targets missing block {}",
                routine_name, target.0
            ),
        ));
        return;
    }
    queue.push_back(target);
}
